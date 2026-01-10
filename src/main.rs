use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha1::{Digest, Sha1};
use std::io::{Read, Result, Write};
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;
use std::{fs, thread};

struct FrameHeader {
    fin: bool,
    opcode: u8,
    payload_len: u64,
    mask: bool,
    masking_key: Option<[u8; 4]>,
    message:Option<Message>,
}

enum Message {
    Text(String),
    Binary(Vec<u8>),
    Ping(String),
    Pong(String),
    Close(String),
    Continue(String),
}


fn send_message(stream: &mut TcpStream, msg: &str) -> Result<()> {
    let payload = msg.as_bytes();

    let mut frame = Vec::new();

    //first byte
    let fin = 1 << 7;
    let opcode = 0x1;
    frame.push(fin | opcode);

    //second byte
    let mask = 0x0;
    let payload_length = payload.len() as u8;
    frame.push(mask | payload_length);

    frame.extend_from_slice(payload);

    let _ = stream.write_all(&frame);
    Ok(())
}

fn read_header(stream: &mut TcpStream) -> Result<FrameHeader> {
    //read the first two bytes which will contain FIN and OPCODE flags and its size will be u8
    let mut first_two = [0u8; 2];
    let _ = stream.read_exact(&mut first_two);

    let byte_one = first_two[0];
    let byte_two = first_two[1];

    //here we are doing AND operation and extraction from the first byte only
    //1 & 0 = 0,1 & 1 =1
    //if fin =1 final frame, else continuous
    let fin = (byte_one & 0b1000_0000) != 0;

    //opcode tells us about type of the message being sent by the client
    let opcode = byte_one & 0b0000_1111;

    // here we will extract from the 2nd byte it will contain if the payload is masked or not and payload
    // if mask = 1 masked else not
    let mask = (byte_two & 0b1000_0000) != 0;
    let mut payload_len = (byte_two & 0b0111_1111) as u64;

    // here we are putting the checks for the payload len
    // normally payload len is 125
    // if it is 126 means the msg is larger and maybe in continuation
    // 127 is the larget len for a payload and takes 8 bytes of the space
    if payload_len == 126 {
        let mut next_two = [0u8; 2];
        let _ = stream.read_exact(&mut next_two);
        payload_len = u16::from_le_bytes(next_two) as u64;
    } else if payload_len == 127 {
        let mut next_eight = [0u8; 8];
        let _ = stream.read_exact(&mut next_eight);
        payload_len = u64::from_le_bytes(next_eight) as u64;
    }

    //here we will get the masking key if the payload is mask
    // this is rule from client to server it should be mask
    // from server to client it shouldn't
    let masking_key = if mask {
        let mut key = [0u8; 4];
        let _ = stream.read_exact(&mut key);
        Some(key)
    } else {
        None
    };

    //now we are converting the masked data to unmasked data
    let mut masked_data = vec![0u8;payload_len as usize];
    let _ = stream.read_exact(&mut masked_data);
    let key = masking_key.unwrap();
    let mut decoded_data = vec![0u8;payload_len as usize];


    if mask {
        for i in 0..payload_len {
            let idx:usize = i.try_into().unwrap();
            decoded_data[idx] = masked_data[idx] ^ key[idx % 4];
        }
    }

    // here we will match all the types opcode can send and accordingly respond back
    let message : Option<Message> = match opcode {
        0x1 => {
           Some(Message::Text(String::from_utf8(decoded_data).expect("undefined byte")))
        }
        0x2 => {
            Some(Message::Binary(decoded_data))
        }
        0x9 => {
            Some(Message::Ping(String::from_str("recieved ping").expect("undefined msg")))
        }
        0xA => {
            Some(Message::Pong(String::from_str("client's pong").expect("undefined msg")))
        }
        0x8 => {
            Some(Message::Close(String::from_str("client's close").expect("undefined msg")))
        }
        0x0 => {
            Some(Message::Continue((String::from_str("Continuation").expect("undefined msg"))))
        }
         _=>None
    };


    Ok(FrameHeader {
        fin,
        opcode,
        payload_len,
        mask,
        masking_key,
        message
    })
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
        loop {
            let read_header = read_header(&mut stream);
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
