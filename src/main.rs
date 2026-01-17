use std::io::Result;
use std::net::TcpListener;
use std::thread;

mod read_header;
mod send_message;
mod handle_client;

use handle_client::handle_client;

pub fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Connection Established");

                thread::spawn(move || {
                    let _ = handle_client(stream);
                });
            }
            Err(e) => {
                println!("Connection failed : {}", e);
            }
        }
    }

    Ok(())
}
