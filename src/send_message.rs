use std::io::{Result, Write};
use std::net::TcpStream;

pub fn send_message(stream: &mut TcpStream, byte1:u8,payload:&[u8]) -> Result<()> {
    let mut frame = Vec::new();

    frame.push(byte1);

    let payload_len = payload.len();

    if payload_len <126 {
        frame.push(payload_len as u8);
    } else if payload_len <= 65535 {
        frame.push(126);
        frame.extend_from_slice(&(payload_len as u16).to_be_bytes());
    } else {
        frame.push(127);
        frame.extend_from_slice(&(payload_len as u64).to_be_bytes());
    }

    frame.extend_from_slice(payload);

    stream.write_all(&frame)

}