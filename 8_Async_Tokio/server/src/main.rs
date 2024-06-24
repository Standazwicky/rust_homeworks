use chrono::Utc;
use shared::{deserialize_message, serialize_message, MessageType};
use std::env;
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::task;
use tracing::{error, info};
use tracing_subscriber;
use anyhow::{Context, Result};
use image::ImageFormat;

async fn handle_client(stream: Arc<Mutex<TcpStream>>) -> Result<MessageType> {
    let mut len_bytes = [0u8; 4];
    {
        let mut stream = stream.lock().await;
        stream.read_exact(&mut len_bytes).await.context("Failed to read message length")?;
        info!("Received message length: {}", u32::from_be_bytes(len_bytes));
    }
    let len = u32::from_be_bytes(len_bytes) as usize;

    let mut buffer = vec![0u8; len];
    {
        let mut stream = stream.lock().await;
        stream.read_exact(&mut buffer).await.context("Failed to read message")?;
        info!("Received message: {:?}", buffer);
    }

    deserialize_message(&buffer).map_err(|e| anyhow::anyhow!(e))
}

async fn handle_message(
    addr: std::net::SocketAddr,
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

async fn report_error(stream: Arc<Mutex<TcpStream>>, error_message: &str) -> Result<()> {
    let error_message = MessageType::Error(error_message.to_string());
    let serialized = serialize_message(&error_message)?;
    let len = serialized.len() as u32;
    {
        let mut stream = stream.lock().await;
        stream.write_all(&len.to_be_bytes()).await.context("Failed to send error length")?;
        stream.write_all(&serialized).await.context("Failed to send error message")?;
        info!("Sent error message: {:?}", error_message);
    }
    Ok(())
}

async fn listen_and_accept(address: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(address).await?;
    info!("Server running on {}", address);

    let clients: Arc<Mutex<HashMap<std::net::SocketAddr, Arc<Mutex<TcpStream>>>>> = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (stream, addr) = listener.accept().await?;
        let stream = Arc::new(Mutex::new(stream));
        clients.lock().await.insert(addr, Arc::clone(&stream));

        let clients = Arc::clone(&clients);
        task::spawn(async move {
            loop {
                match handle_client(Arc::clone(&stream)).await {
                    Ok(message) => {
                        match handle_message(addr, message).await {
                            Ok(true) => {
                                // Client sent a Quit message
                                info!("Client {} disconnected", addr);
                                break;
                            }
                            Ok(false) => {
                                // Continue to handle other messages
                            }
                            Err(e) => {
                                error!("Error handling message from {}: {:?}", addr, e);
                                if let Err(report_err) = report_error(Arc::clone(&stream), &e.to_string()).await {
                                    error!("Failed to report error to client {}: {:?}", addr, report_err)
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error handling client {}: {:?}", addr, e);
                        if let Err(report_err) = report_error(Arc::clone(&stream), &e.to_string()).await {
                            error!("Failed to report error to client {}: {:?}", addr, report_err)
                        }
                        break;
                    }
                }
            }
            // Remove the client from the map when done
            clients.lock().await.remove(&addr);
        });
    }
}

fn ensure_directories_exist() -> Result<()> {
    let paths = ["images", "files"];
    for path in &paths {
        if !Path::new(path).exists() {
            fs::create_dir(path).context(format!("Failed to create directory {}", path))?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
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

    if let Err(e) = listen_and_accept(address).await {
        error!("Error: {}", e);
    }
}
