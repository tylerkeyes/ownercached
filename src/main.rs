pub mod data;
pub mod logging;

use bytes::Bytes;
use chrono::{Duration, Utc};
use clap::Parser;
use core::str;
use data::{DataStore, StoredValue};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    str::SplitWhitespace,
    string::String,
    sync::Arc,
    thread,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'p', long = "port", default_value_t = 11211)]
    port: usize,
}

fn main() {
    let args = Args::parse();
    let port = args.port;
    let address = format!("127.0.0.1:{}", port);

    let listener = TcpListener::bind(address).unwrap();
    info!("starting server on ::{}", port);

    let data_store = DataStore::new();
    let data_store_arc = Arc::new(data_store);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let data_store_clone = Arc::clone(&data_store_arc);
                thread::spawn(move || {
                    connection_handler(stream, data_store_clone);
                });
            }
            Err(e) => {
                error!("Connection failed: {}", e);
            }
        }
    }
}

fn connection_handler(mut stream: TcpStream, data_store: Arc<DataStore>) {
    info!(
        "received request from client: {}",
        stream.peer_addr().unwrap()
    );
    let mut buffer = [0; 1024];

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                let client = stream.peer_addr().unwrap();
                info!("Client disconnected: {}", client);
                break;
            }
            Ok(n) => {
                let command = String::from_utf8_lossy(&buffer[..n]).into_owned();
                let mut command_items = command.split_whitespace();
                let action = command_items.next().unwrap();
                let key = command_items.next().unwrap();

                match action {
                    "set" => {
                        let stored_value = input_handler(&mut stream, &mut command_items);
                        data_store.set(String::from(key), stored_value);

                        let noreply = command_items.next();
                        if noreply.is_none() || noreply.unwrap() != "noreply" {
                            let response = "STORED\r\n";
                            _ = stream.write(response.as_bytes()).unwrap();
                        }
                    }
                    "get" => {
                        let stored_item = data_store.get(String::from(key));
                        let response = match stored_item {
                            Some(v) => {
                                if is_expired(v.exptime) {
                                    data_store.remove(String::from(key));
                                    "END\r\n".to_string()
                                } else {
                                    v.response_string(key)
                                }
                            }
                            None => "END\r\n".to_string(),
                        };
                        _ = stream.write(response.as_bytes()).unwrap();
                    }
                    "add" => {
                        let stored_value = input_handler(&mut stream, &mut command_items);

                        let response = if !data_store.contains(String::from(key)) {
                            data_store.set(String::from(key), stored_value);
                            "STORED\r\n"
                        } else {
                            "NOT_STORED\r\n"
                        };

                        let noreply = command_items.next();
                        if noreply.is_none() || noreply.unwrap() != "noreply" {
                            _ = stream.write(response.as_bytes()).unwrap();
                        }
                    }
                    "replace" => {
                        let stored_value = input_handler(&mut stream, &mut command_items);

                        let response = if data_store.contains(String::from(key)) {
                            data_store.set(String::from(key), stored_value);
                            "STORED\r\n"
                        } else {
                            "NOT_STORED\r\n"
                        };

                        let noreply = command_items.next();
                        if noreply.is_none() || noreply.unwrap() != "noreply" {
                            _ = stream.write(response.as_bytes()).unwrap();
                        }
                    }
                    "append" => {
                        let stored_value = input_handler(&mut stream, &mut command_items);

                        let response = if data_store.contains(String::from(key)) {
                            let old = data_store.get(String::from(key)).unwrap();
                            data_store.append(String::from(key), old, stored_value);
                            "STORED\r\n"
                        } else {
                            "NOT_STORED\r\n"
                        };

                        let noreply = command_items.next();
                        if noreply.is_none() || noreply.unwrap() != "noreply" {
                            _ = stream.write(response.as_bytes()).unwrap();
                        }
                    }
                    "prepend" => {
                        let stored_value = input_handler(&mut stream, &mut command_items);

                        let response = if data_store.contains(String::from(key)) {
                            let old = data_store.get(String::from(key)).unwrap();
                            data_store.prepend(String::from(key), old, stored_value);
                            "STORED\r\n"
                        } else {
                            "NOT_STORED\r\n"
                        };

                        let noreply = command_items.next();
                        if noreply.is_none() || noreply.unwrap() != "noreply" {
                            _ = stream.write(response.as_bytes()).unwrap();
                        }
                    }
                    _ => {
                        let response =
                            format!("HTTP/1.1 500 ERR\r\nNot a valid command: {}\r\n", action);
                        _ = stream.write(response.as_bytes()).unwrap();
                    }
                }
            }
            Err(e) => {
                error!("Failed to write to client: {}", e);
                break;
            }
        }
        stream.flush().unwrap();

        // inspect the data_store
        debug!("Data Store:");
        for entry in data_store.iter() {
            debug!("\t- {}:\n\t  {:?}", entry.key(), entry.value());
        }
    }
}

fn input_handler(stream: &mut TcpStream, command_items: &mut SplitWhitespace<'_>) -> StoredValue {
    let mut stored_value = StoredValue::new();
    stored_value.set_flags(command_items.next().unwrap().parse::<u16>().unwrap());

    let exptime = command_items.next().unwrap().parse::<isize>().unwrap();
    let end = match exptime {
        n if n < 0 => n,
        0 => 0,
        _ => (Utc::now() + Duration::seconds(exptime as i64)).timestamp() as isize,
    };
    stored_value.set_exptime(end);

    stored_value.set_byte_count(command_items.next().unwrap().parse::<usize>().unwrap());

    let bytes = stored_value.get_byte_count();
    // assume the value will be smaller than this
    let mut buffer = [0; 1024];
    _ = stream.read(&mut buffer).unwrap();

    let value = Bytes::copy_from_slice(&buffer[..bytes]);
    stored_value.set_bytes(value);

    stored_value
}

fn is_expired(exptime: isize) -> bool {
    match exptime {
        n if n < 0 => true,
        0 => false,
        _ => {
            let now = (Utc::now()).timestamp() as isize;
            now > exptime
        }
    }
}
