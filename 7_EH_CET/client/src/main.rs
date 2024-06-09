use shared::{serialize_message, MessageType};
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::path::Path;
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

// Function to start the client and connect to the server
fn start_client(address: &str) -> Result<()> {
    let stream = TcpStream::connect(address).context("Failed to connect to server")?;
    info!("Connected to server at {}", address);

    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input = input.trim();

        // Check if the input starts with specific commands
        if input.starts_with(".file") {
            let path = &input[6..];
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
                send_message(&stream, &message)?;
            } else {
                eprintln!("Failed to open file {}", path);
            }
        } else if input.starts_with(".image ") {
            let path = &input[7..];
            if let Ok(mut file) = File::open(&Path::new(path)) {
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).context("Failed to read image")?;
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
