use chrono::Utc;
use shared::{deserialize_message, serialize_message, MessageType};
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use tracing::{error, info};
use tracing_subscriber;
use anyhow::{Context, Result};
use image::ImageFormat;

// Function to handle incoming client connections
pub fn handle_client(mut stream: TcpStream) -> Result<MessageType> {
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes).context("Failed to read message length")?;
    let len = u32::from_be_bytes(len_bytes) as usize;

    let mut buffer = vec![0u8; len];
    stream.read_exact(&mut buffer).context("Failed to read message")?;

    deserialize_message(&buffer).map_err(|e| anyhow::anyhow!(e))
}

// Function to handle different types of messages from clients
pub fn handle_message(
    addr: SocketAddr,
    message: MessageType,
) -> Result<bool> {
    match message {
        MessageType::Quit => {
            info!("Client {} sent quit message", addr);
            return Ok(true);
        }
        MessageType::Text(text) => {
            info!("Text message from {}: {}", addr, text);
        }
        MessageType::Image(data) => {
            ensure_directories_exist()?;
            info!("Receiving image from {}...", addr);
            let timestamp = Utc::now().timestamp();
            let filename = format!("images/{}.png", timestamp);
            
            // Convert image to PNG format
            let image = image::load_from_memory(&data).context("Failed to load image from memory")?;
            let mut output_file = File::create(&filename).context("Failed to create image file")?;
            image.write_to(&mut output_file, ImageFormat::Png).context("Failed to write image as PNG")?;
            
            info!("Saved image to {}", filename);
        }
        MessageType::File(name, data) => {
            ensure_directories_exist()?;
            info!("Receiving file '{}' from {}...", name, addr);
            let filename = format!("files/{}", name);
            fs::write(&filename, &data).context("Failed to save file")?;
            info!("Saved file to {}", filename);
        }
        MessageType::Error(err) => {
         error!("Error from {}: {}", addr, err);   
        }
    }
    Ok(false)
}

fn report_error(mut stream: TcpStream, error_message: &str) -> Result<()> {
 let error_message=MessageType::Error(error_message.to_string());
 let serialized = serialize_message(&error_message)?;
 let len = serialized.len() as u32;
 stream.write(&len.to_be_bytes()).context("Failed to send error length")?;
 stream.write_all(&serialized).context("Failed to send error message")?;
 Ok(())
}


// Function to start the server and listen for incoming connections
pub fn listen_and_accept(address: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(address)?;
    info!("Server running on {}", address);

    let clients: Arc<Mutex<HashMap<SocketAddr, TcpStream>>> = Arc::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let addr = stream.peer_addr().unwrap();
        clients
            .lock()
            .unwrap()
            .insert(addr.clone(), stream.try_clone().unwrap());

        let clients = Arc::clone(&clients);
        thread::spawn(move || {
            loop {
                match handle_client(stream.try_clone().unwrap()) {
                    Ok(message) => {
                        match handle_message(addr, message) {
                            Ok(true) => {
                                //Client sent a Quit message
                                break;
                            }
                            Ok(false) => {
                                // Continue to handle other messages
                            }
                            Err(e) => {
                                error!("Error handling message from {}: {:?}", addr, e);
                                if let Err(report_err) = report_error(stream.try_clone().unwrap(), &e.to_string()) {
                                    error!("Failed to report error to client {}: {:?}", addr, report_err)
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error handling client {}: {:?}", addr, e);
                        if let Err(report_err) = report_error(stream.try_clone().unwrap(), &e.to_string()) {
                                    error!("Failed to report error to client {}: {:?}", addr, report_err)
                                }
                        break;
                    }
                }
            }
            // Remove the client from the map when done
            clients.lock().unwrap().remove(&addr);
        });
    }

    Ok(())
}

// Function to ensure that necessary directories exist
fn ensure_directories_exist() -> Result<()> {
    let paths = ["images", "files"];
    for path in &paths {
        if !Path::new(path).exists() {
            fs::create_dir(path).context(format!("Failed to create directory {}", path))?;
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
        println!("Setting default: localhost:11111");
        "localhost:11111"
    } else {
        &args[1]
    };

    if let Err(e) = listen_and_accept(address) {
        error!("Error: {}", e);
    }
}
