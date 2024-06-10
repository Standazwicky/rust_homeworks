use shared::{deserialize_message, serialize_message, MessageType};
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::thread;
use tracing::{error, info};
use tracing_subscriber;
use anyhow::{Context, Result};

// Function to send a message to the server
fn send_message(
    mut stream: &TcpStream,
    message: &MessageType,
) -> Result<()> {
    let serialized = serialize_message(message).map_err(|e| anyhow::anyhow!(e))?;

    // Send the length of the serialized message (as 4-byte value).
    let len = serialized.len() as u32;
    stream.write(&len.to_be_bytes()).context("Failed to send message length")?;
    // Send the serialized message.
    stream.write_all(&serialized).context("Failed to send message")?;
    Ok(())
}

fn handle_server_response(mut reader: TcpStream) -> Result<()> {
    loop {
        let mut len_bytes = [0u8;4];
        reader.read_exact(&mut len_bytes).context("Failed to read response length")?;
        let len = u32::from_be_bytes(len_bytes) as usize;
        
        let mut buffer = vec![0u8; len];
        reader.read_exact(&mut buffer).context("Failed to read response")?;
        
        let message = deserialize_message(&buffer).map_err(|e| anyhow::anyhow!(e))?;
        match message {
          MessageType::Error(err) => {
              error!("Error from server: {}", err);
          }
          MessageType::Text(text) => {
              info!("Server response: {}", text);
          }
          _ => {
              info!("Received unexpected message from server");
          }
        }
      }
}
     

// Function to start the client and connect to the server
fn start_client(address: &str) -> Result<()> {
    let stream = TcpStream::connect(address).context("Failed to connect to server")?;
    info!("Connected to server at {}", address);
    
    let reader = stream.try_clone().context("Failed to clone stream for reading")?;
    let writer = stream;

    thread::spawn(move || {
        if let Err(e) = handle_server_response(reader) {
            error!("Error handling server response: {}",e);
        }
    });
    
    // List of valid commands
    let valid_commands = [".file", ".image", ".quit"];
    
    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input = input.trim();

        // Check if the input is command
        if input.starts_with('.') {
            let command = input.split_whitespace().next().unwrap_or("");
            
            // Check if command is valid
            if !valid_commands.contains(&command) {
                eprintln!("Invalid command. Valid commands are: .file <path>, .image <path>, .quit");
                continue;
            }
        
        match command  {
        ".file" => {
            if input.len() <= 6 {
                eprintln!("Error: .file command requires a file path.");
                continue;
            }
            let path = &input[6..].trim();
            if let Ok(mut file) = File::open(&Path::new(path)) {
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).context("Failed to read file")?;
                let filename = Path::new(path)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                let message = MessageType::File(filename, buffer);
                send_message(&writer, &message)?;
            } else {
                eprintln!("Failed to open file {}", path);
            }
        } 
        ".image" => {
            if input.len() <= 7 {
             eprintln!("Error: .image command requires a file path.");
             continue;
            }
            let path = &input[7..].trim();
            if let Ok(mut file) = File::open(&Path::new(path)) {
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).context("Failed to read image")?;
                let message = MessageType::Image(buffer);
                send_message(&writer, &message)?;
            } else {
                eprintln!("Failed to open image {}", path);
            }
        } 
        ".quit" => {
            let message = MessageType::Quit;
            send_message(&writer, &message)?;
            break;
        }
        _ => {}
        }  
        
    } else {
            let message = MessageType::Text(input.to_string());
            send_message(&writer, &message)?;
        }
      }
    Ok(())    
    }
    

fn main() {
    // Initialize the tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args: Vec<String> = env::args().collect();

    let address = if args.len() < 2 {
        println!("Usage: {} <address>", args[0]);
        println!("Setting default: localhost:1111");
        "localhost:11111"
    } else {
        &args[1]
    };

    if let Err(e) = start_client(address) {
        error!("Error: {}", e);
    }
}
