use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha1::{Digest, Sha1};
use std::io::{Read, Result, Write};
use std::net::{TcpListener, TcpStream};
use std::{fs, thread};

fn send_message(stream: &mut TcpStream,msg:&str) -> Result<()>{

    let payload = msg.as_bytes();
    let payload_len = payload.len();

    let mut frame = Vec::new();

    frame.push(0x81);
    frame.push(payload_len as u8);
    frame.extend_from_slice(payload);

    let _ = stream.write_all(&frame);
    Ok(())
}

fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut buffer = [0; 512];

    let bytes_read = stream.read(&mut buffer).expect("Failed to read stream");

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);

    if request.contains("Upgrade: websocket") {
        let mut websocket_key = String::new();
        for line in request.lines() {
            if line.starts_with("Sec-WebSocket-Key:") {
                websocket_key = Some(line.split(":").nth(1).unwrap().trim().to_string()).unwrap();
            }
        }

        let mut hasher = Sha1::new();
        hasher.update(format!(
            "{}258EAFA5-E914-47DA-95CA-C5AB0DC85B11",
            websocket_key
        ));
        let hashed_key = hasher.finalize();
        let accepted_key = STANDARD.encode(hashed_key);

        stream.write_all(
            format!(
                "HTTP/1.1 101 Switching Protocols\r\n\
                Upgrade: websocket\r\n\
                Connection: Upgrade\r\n\
                Sec-WebSocket-Accept: {}\r\n\r\n",
                accepted_key
            )
            .as_bytes(),
        )?;
        loop{
            let mut buf = [0;512];

            send_message(&mut stream, "hola");
        }
    }

    let contents = fs::read("src/index.html").unwrap();

    stream.write_all(b"HTTP/1.1 200 OK\r\n")?;
    stream.write_all(b"Content-Type: text/html\r\n")?;
    stream.write_all(format!("Content-Length: {}\r\n\r\n", contents.len()).as_bytes())?;
    stream.write_all(&contents)?;

    Ok(())
}

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
