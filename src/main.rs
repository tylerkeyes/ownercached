pub mod data;

use clap::Parser;
use core::str;
use data::StoredValue;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    str::SplitWhitespace,
    string::String,
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
    println!("starting server on ::{}", port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    connection_handler(stream);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

fn connection_handler(mut stream: TcpStream) {
    println!("received request");
    let mut buffer = [0; 1024];

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Client disconnected");
                break;
            }
            Ok(n) => {
                let command = String::from_utf8_lossy(&buffer[..n]).into_owned();
                let mut command_items = command.split_whitespace();
                let action = command_items.next().unwrap();
                println!("{:?}", action);

                match action {
                    "set" => {
                        println!("handle set");
                        let (stored_value, key) = set_handler(&mut stream, command_items);
                        println!("StoredValue: {:?}, key: {}", stored_value, key);
                    }
                    _ => {
                        let response =
                            format!("HTTP/1.1 500 ERR\r\nNot a valid command: {}\r\n", action);
                        _ = stream.write(response.as_bytes()).unwrap();
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to write to client: {}", e);
                break;
            }
        }
        stream.flush().unwrap();
    }
}

fn set_handler(
    stream: &mut TcpStream,
    mut command_items: SplitWhitespace<'_>,
) -> (StoredValue, String) {
    let key = command_items.next().unwrap();
    let mut stored_value = StoredValue::new();
    stored_value.set_flags(command_items.next().unwrap().parse::<u16>().unwrap());
    stored_value.set_exptime(command_items.next().unwrap().parse::<usize>().unwrap());
    stored_value.set_byte_count(command_items.next().unwrap().parse::<usize>().unwrap());

    let bytes = stored_value.get_byte_count();
    // assume it will be smaller than this
    let mut buffer = [0; 1024];
    _ = stream.read(&mut buffer).unwrap();

    (stored_value, String::from(key))
}
