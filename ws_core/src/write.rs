use std::io::{Result, Write};
use std::net::TcpStream;

pub fn send_server_message(stream: &mut TcpStream, byte1:u8,payload:&[u8]) -> Result<()> {
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

pub fn send_client_message(stream: &mut TcpStream, byte1:u8,payload:&[u8],masking_key : [u8;4]) -> Result<()> {
    let mut frame = Vec::new();

    frame.push(byte1);

    let mask = 0b1000_0000;
    let payload_len = payload.len();
    let byte2 :u8;

    if payload_len <126 {
        byte2 = (mask | payload_len) as u8;
        frame.push(byte2);
    } else if payload_len <= 65535 {
        byte2 = (mask | 126) as u8;
        frame.push(byte2);
        frame.extend_from_slice(&(payload_len as u16).to_be_bytes());
    } else {
        byte2 = (mask | 127) as u8;
        frame.push(byte2);
        frame.extend_from_slice(&(payload_len as u64).to_be_bytes());
    }

    frame.extend_from_slice(&masking_key);

    frame.extend_from_slice(payload);

    stream.write_all(&frame)

}