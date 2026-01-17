use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha1::{Digest, Sha1};
use std::io::{Read, Result, Write};
use std::net::TcpStream;


use crate::read_header::read_header;

pub fn handle_client(mut stream: TcpStream) -> Result<()> {
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

        loop {
            let _read_frame = read_header(&mut stream);
        }
    }
    stream.write_all(b"HTTP/1.1 200 OK\r\n")?;
    stream.write_all(b"Content-Type: text/html\r\n")?;
    Ok(())
}