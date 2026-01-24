use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha1::{Digest, Sha1};
use std::io::{Read, Result, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use ws_core::read::read_header;
use ws_core::write::send_message;


pub fn handle_client(mut stream: TcpStream) -> Result<()> {
    // these variable are here to read all the buffer sent from the client
    let mut buffer = Vec::new();
    let mut temp = [0u8;1024];

    loop{
        let bytes_read = stream.read(&mut temp).expect("failed to read stream");
        //break the connection here if no bytes read
        if bytes_read == 0{
            break;
        }

        //merge the bytes read in the buffer
        buffer.extend_from_slice(&temp[..bytes_read]);

        //if we find any bytes \r\n\r\n , stop reading more frames
        // \r\n\r\n = CR LF CR LF
        // CR = Carriage Line  = \r = ASCII 13
        // LF = Line feed  = \n = ASCII 10
        // in HTTP protocol this is considered to use when we have to end any message.
        if buffer.windows(4).any(|w| w == b"\r\n\r\n"){
            break;
        }
    }

    //convert the buffer byte to string
    let request = String::from_utf8_lossy(&buffer);

    //check if the request contains the upgrade websocket
    if request.contains("Upgrade: websocket") {
        let mut websocket_key = String::new();
        //get the key to upgrade the protocol
        for line in request.lines() {
            if line.starts_with("Sec-WebSocket-Key:") {
                websocket_key = line.split(":").nth(1).unwrap().trim().to_string();
            }
        }

        // use sha1 to encrypt the key and send it back to the client
        let mut hasher = Sha1::new();
        hasher.update(format!(
            "{}258EAFA5-E914-47DA-95CA-C5AB0DC85B11",
            websocket_key
        ));
        let hashed_key = hasher.finalize();
        let accepted_key = STANDARD.encode(hashed_key);

        // use the 101 status code to upgrade the connection
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

        //open the loop to read the streams in websocket protocol
        loop {
            let fin_code = 0b1000_0000;
            let mut final_message = Vec::new();
            let mut is_first = true;
            let mut opcode: u8 = 0;

            loop {
                //read each from coming from the the client
                let frame = read_header(&mut stream).unwrap();
                let cont_opcode = frame.opcode;

                //close frame from the client
                if cont_opcode == 0b0000_1000 {
                    let byte1 = fin_code | 0b0000_1000;
                    send_message(&mut stream, byte1, &frame.decoded_data)?;
                    return Ok(());
                }

                //ping from the client
                if cont_opcode == 0b0000_1001 {
                    let byte1 = fin_code | 0b0000_1010;
                    send_message(&mut stream, byte1, &frame.decoded_data)?;
                    continue;
                }

                //pong from the client
                if cont_opcode == 0b0000_1010 {
                    continue;
                }

                //set opcode from the first frame as it changes from the current form to another as new frame comes
                //1st frame tells the type , 2nd frame 0 untill fin flag become 1
                if is_first {
                    opcode = cont_opcode;
                    is_first = false;
                }

                final_message.extend(frame.decoded_data); //extend the final message untill the final frame comes
                if frame.fin {
                    break; //break the loop when final frame comes
                }
            }

            //handle the TEXT and BINARY opcode types
            //TEXT type opcode = 1 (0b0000_0001)
            //BINARY type opcode = 2 (0b0000_0010)
            if opcode == 0b0000_0001 {
                if let Ok(text) = String::from_utf8(final_message.clone()){
                    println!("Text recieved : {}",text);

                    let byte1 = fin_code | opcode;
                    send_message(&mut stream, byte1, text.as_bytes())?;
                    continue;
                }
            } else {
                println!("Recieved {} of binary message",final_message.len());

                let byte1 = fin_code | opcode;
                send_message(&mut stream, byte1, &final_message)?;
                continue;
            }
        }
    }
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