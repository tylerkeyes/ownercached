#[macro_export]
macro_rules! debug {
    ($fmt:expr, $($arg:tt)*) => {
        {
            use std::env;
            if let Ok(log_level) = env::var("LOG_LEVEL") {
                if log_level.to_lowercase() == "debug" {
                    println!("[DEBUG] {}", format!($fmt, $($arg)*));
                }
            }
        }
    };
    ($msg:expr) => {
        {
            use std::env;
            if let Ok(log_level) = env::var("LOG_LEVEL") {
                if log_level.to_lowercase() == "debug" {
                    println!("[DEBUG] {}", $msg);
                }
            }
        }
    };
}

#[macro_export]
macro_rules! info {
    ($fmt:expr, $($arg:tt)*) => {
        {
            println!("[INFO] {}", format!($fmt, $($arg)*));
        }
    };
    ($msg:expr) => {
        {
            println!("[INFO] {}", $msg);
        }
    };
}

#[macro_export]
macro_rules! error {
    ($fmt:expr, $($arg:tt)*) => {
        {
            println!("[ERROR] {}", format!($fmt, $($arg)*));
        }
    };
    ($msg:expr) => {
        {
            println!("[ERROR] {}", $msg);
        }
    };
}
