use std::io::{Result, Write};
use std::net::TcpStream;

pub fn send_message(stream: &mut TcpStream, msg: &str) -> Result<()> {
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