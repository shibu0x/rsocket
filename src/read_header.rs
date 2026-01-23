use std::io::{Read, Result};
use std::net::TcpStream;

#[derive(Debug,Clone)]
pub struct FrameHeader {
    pub fin: bool,
    pub opcode: u8,
    pub decoded_data : Vec<u8>,
}

pub fn read_header(stream: &mut TcpStream) -> Result<FrameHeader> {
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
        payload_len = u16::from_be_bytes(next_two) as u64;
    } else if payload_len == 127 {
        let mut next_eight = [0u8; 8];
        let _ = stream.read_exact(&mut next_eight);
        payload_len = u64::from_be_bytes(next_eight) as u64;
    }

    //now we will get the masking key if we get mask as 1 means the payload data is masked
    // masking key comes after the payload len bits
    let masking_key = if mask {
        let mut key = [0u8;4];
        stream.read_exact(&mut key).expect("unable to read masking key");
        Some(key)
    } else{
        None
    };
    let mut decoded_data = vec![0u8;payload_len as usize];

    //check if the mask is present then decode the payload data and then unmask it using the masking key 
    if mask {
        let mut payload_data = vec![0u8;payload_len as usize];
        stream.read_exact(&mut payload_data).expect("unable to read payload_data");
        let key = masking_key.unwrap();
        for i in 0..payload_len {
            let idx: usize = i.try_into().unwrap();
            decoded_data[idx] = payload_data[idx] ^ key[idx % 4];
        }
    } else {
        stream.read_exact(&mut decoded_data).expect("unable to read payload_data");
    }


    Ok(FrameHeader {
        fin,
        opcode,
        decoded_data,
    })
}
