use std::io::{Read, Result, Write};
use std::net::TcpStream;

use base64::Engine;
use base64::engine::general_purpose;
use rand::Rng;
use sha1::{Digest, Sha1};
use ws_core::write::send_client_message;

pub fn handle_server(mut stream: TcpStream) -> Result<()> {
    let random_bs64_key = generate_base64_key(16);
    //send a http request to the server for upgrading the protocol from normal http to websocket
    stream.write_all(
        format!(
            "GET / HTTP/1.1\r\n\
             Host: 127.0.0.1:8080\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Key: {}\r\n\
             Sec-WebSocket-Version: 13\r\n\r\n",
            random_bs64_key
        )
        .as_bytes(),
    )?;

    let mut buffer = Vec::new();
    let mut temp = [0u8; 1024];

    //loop over all the streams and stop when we get CL RF CL RF
    // this makes us read all the valid stream and stops us from breaking and reading only half streamed data
    loop {
        let bytes_read = stream.read(&mut temp).expect("failed to read the stream");
        if bytes_read == 0 {
            break;
        }

        buffer.extend_from_slice(&temp[..bytes_read]);

        // same as server side
        if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
    }

    let request = String::from_utf8_lossy(&buffer);

    //check if the server accepted the connection
    if request.contains("Upgrade: websocket")
        && request.contains("Connection: Upgrade")
        && request.contains("Sec-WebSocket-Accept")
    {
        let mut server_key = String::new();

        //get the server key which server sent for upgrading
        for line in request.lines() {
            if line.starts_with("Sec-WebSocket-Accept:") {
                server_key = line.split(":").nth(1).unwrap().trim().to_string();
            }
        }

        //expected key = base64 (SHA1 (random base 64 key + Magic UID))
        let mut hasher = Sha1::new();
        hasher.update(format!(
            "{}258EAFA5-E914-47DA-95CA-C5AB0DC85B11",
            random_bs64_key
        ));
        let hashed_key = hasher.finalize();
        let received_key = general_purpose::STANDARD.encode(hashed_key);

        //match the expected key with the server key if yes then create a loop untill the connection closes
        if received_key == server_key {
            loop {
                let msg = "Hello, server".as_bytes();
                let fincode = 0b1000_0000;
                let opcode = 0b0000_0001;

                let byte1 = fincode | opcode;

                let mut masking_key = [0u8; 4];
                rand::rng().fill(&mut masking_key[..]);
                let mut encoded_data = vec![0u8; msg.len()];

                for i in 0..msg.len() {
                    let idx: usize = i.try_into().unwrap();
                    encoded_data[idx] = msg[idx] ^ masking_key[idx % 4];
                }

                send_client_message(&mut stream, byte1, &encoded_data, masking_key)?;
                break;
            }
        } else {
            println!("failed to upgrade connection")
        }
    }

    Ok(())
}

pub fn main() -> Result<()> {
    //connect the client with the servers tcp connection
    match TcpStream::connect("127.0.0.1:8080") {
        Ok(stream) => {
            println!("Connected Successfully.");
            handle_server(stream)?;
        }
        Err(err) => {
            println!("Error Connecting Server : {}", err);
        }
    }
    Ok(())
}

// function to create a base 64 key and it is done because the server accepts 16 bytes of base 64 encoded key to upgrade the protocol
fn generate_base64_key(bytes: u8) -> String {
    let mut key_bytes = vec![0u8; bytes as usize];

    rand::rng().fill(&mut key_bytes[..]);

    let encoded = general_purpose::STANDARD.encode(&key_bytes);

    encoded
}
