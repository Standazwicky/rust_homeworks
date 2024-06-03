use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::env;
use std::fs::File;
use std::path::Path;
use shared::{MessageType, serialize_message};

fn send_message(
    mut stream: &TcpStream,
    message: &MessageType,
) -> Result<(), Box<dyn std::error::Error>> {
    let serialized = serialize_message(message);

    // Send the length of the serialized message (as 4-byte value).
    let len = serialized.len() as u32;
    stream.write(&len.to_be_bytes())?;

    // Send the serialized message.
    stream.write_all(&serialized)?;
    Ok(())
}

fn start_client(address: &str) -> Result<(), Box<dyn std::error::Error>> {
    let stream = TcpStream::connect(address)?;
    println!("Connected to server at {}", address);

    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input = input.trim();

        if input.starts_with(".file") {
            let path = &input[6..];
            if let Ok(mut file) = File::open(&Path::new(path)) {
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                let filename = Path::new(path)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                let message = MessageType::File(filename, buffer);
                send_message(&stream, &message)?;
            } else {
                eprintln!("Failed to open file {}", path);
            }
        } else if input.starts_with(".image ") {
            let path = &input[7..];
            if let Ok(mut file) = File::open(&Path::new(path)) {
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                let message = MessageType::Image(buffer);
                send_message(&stream, &message)?;
            } else {
                eprintln!("Failed to open image {}", path);
            }
        } else if input == ".quit" {
            let message = MessageType::Quit;
            send_message(&stream, &message)?;
            break;
        } else {
            let message = MessageType::Text(input.to_string());
            send_message(&stream, &message)?;
        }
    }
    Ok(())
}

fn main() {
 let args: Vec<String> = env::args().collect();
 
 let address = if args.len() < 2 {
     "localhost:11111"   
    } else {
     &args[1]
    };
    
    if let Err(e) = start_client(address) {
     eprintln!("Error: {}",e);   
    }
}
