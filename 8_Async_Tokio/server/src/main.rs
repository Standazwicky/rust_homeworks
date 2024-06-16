use anyhow::{Context, Result};
use chrono::Utc;
use shared::{deserialize_message, serialize_message, MessageType};
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::task;
use tracing::{error, info};
use tracing_subscriber;

 type SharedStream = Arc<Mutex<TcpStream>>;
 type ClientMap = Arc<Mutex<HashMap<SocketAddr, SharedStream>>>;

// Function to handle incoming client connections
async fn handle_client(stream: SharedStream) -> Result<MessageType> {
    let mut stream = stream.lock().await;
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes).await.context("Failed to read message length")?;
    let len = u32::from_be_bytes(len_bytes) as usize;
    let mut buffer = vec![0u8; len];
    stream.read_exact(&mut buffer).await.context("Failed to read message")?;
    let message = deserialize_message(&buffer).map_err(|e| anyhow::anyhow!(e))?;
    Ok(message)
}

async fn send_message(stream: SharedStream, message: &MessageType) -> Result<()> {
  let mut stream = stream.lock().await;
  let serialized = serialize_message(message).map_err(|e| anyhow::anyhow!(e))?;
  let len = serialized.len() as u32;
  stream.write_all(&len.to_be_bytes()).await.context("Failed to send message lenght")?;
  stream.write_all(&serialized).await.context("Failed to send message")?;
  Ok(())
}

// Function to handle different types of messages from clients
async fn handle_message(
    addr: SocketAddr,
    message: MessageType
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
            ensure_directories_exist().await?;
            info!("Receiving image from {}...", addr);
            let timestamp = Utc::now().timestamp();
            let filename = format!("images/{}.png", timestamp);
            
            // Convert image to PNG format and save
            let image = image::load_from_memory(&data).context("Failed to load image from memory")?;
            image.save(&filename).context("Failed to save image as PNG")?;
            
            info!("Saved image to {}", filename);
        }
        MessageType::File(name, data) => {
            ensure_directories_exist().await?;
            info!("Receiving file '{}' from {}...", name, addr);
            let filename = format!("files/{}", name);
            tokio::fs::write(&filename, &data).await.context("Failed to save file")?;
            info!("Saved file to {}", filename);
        }
        MessageType::Error(err) => {
         error!("Error from {}: {}", addr, err);   
        }
    }
    Ok(false)
}

async fn report_error(stream: SharedStream, error_message: &str) -> Result<()> {
 let error_message=MessageType::Error(error_message.to_string());
 send_message(stream, &error_message).await
}


// Function to start the server and listen for incoming connections
async fn listen_and_accept(address: &str, clients: ClientMap) -> std::io::Result<()> {
    let listener = TcpListener::bind(address).await?;
    info!("Server running on {}", address);
    
    loop {
        let (stream, addr) = listener.accept().await?;
        let stream = Arc::new(Mutex::new(stream));
        
        clients.lock().await.insert(addr, stream.clone());
        
        let clients = clients.clone();
        task::spawn(async move {
            loop {
               match handle_client(stream.clone()).await {
                   Ok(message) => {
                        match handle_message(addr, message).await {
                            Ok(true) => {
                                //Client sent a Quit message
                                info!("Client {} disconnected.", addr);
                                break;
                            }
                            Ok(false) => {
                                // Continue to handle other messages
                            }
                            Err(e) => {
                                error!("Error handling message from {}: {:?}", addr, e);
                                if let Err(report_err) = report_error(stream.clone(), &e.to_string()).await {
                                    error!("Failed to report error to client {}: {:?}", addr, report_err);
                                }
                            }
                        }
                 }
                 Err(e) => {
                        error!("Error handling client {}: {}", addr, e);
                        if let Err(report_err) = report_error(stream.clone(), &e.to_string()).await {
                                    error!("Failed to report error to client {}: {:?}", addr, report_err)
                                }
                        break;
                    }
                }
            }
            clients.lock().await.remove(&addr);
        });    
      }
 }

// Function to ensure that necessary directories exist
async fn ensure_directories_exist() -> Result<()> {
    let paths = ["images", "files"];
    for path in &paths {
        if !Path::new(path).exists() {
            tokio::fs::create_dir(path).await.context(format!("Failed to create directory {}", path))?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
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
    
    let clients: ClientMap = Arc::new(Mutex::new(HashMap::new()));

    if let Err(e) = listen_and_accept(address, clients).await {
        error!("Error: {}", e);
    }
}
