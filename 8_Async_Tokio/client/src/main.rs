use shared::{deserialize_message, serialize_message, MessageType};
use std::env;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio::task;
use tracing::{error, info};
use tracing_subscriber;
use anyhow::{Context, Result};

// Main function    
#[tokio::main]
async fn main() {
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

    if let Err(e) = start_client(address).await {
        error!("Error: {}", e);
    }
}


// Function to start the client and connect to the server
async fn start_client(address: &str) -> Result<()> {
    let stream = TcpStream::connect(address).await.context("Failed to connect to server")?;
    info!("Connected to server at {}", address);
    
    let (reader, writer) = stream.into_split();
    let reader = Arc::new(Mutex::new(reader));
    let writer = Arc::new(Mutex::new(writer));
    
    let (tx, mut rx) = mpsc::channel::<MessageType>(100);
    
    // Task for handling server responses
    let reader_clone = Arc::clone(&reader);
    task::spawn(async move {
        if let Err(e) = handle_server_response(reader_clone, tx).await {
            error!("Error handling server response: {}",e);
        }
    });
    
    // Task for handling user input
    let writer_clone = Arc::clone(&writer);
    task::spawn(async move { 
        if let Err(e) = handle_user_input(writer_clone).await {
            error!("Error handling user input: {}", e);
        }
    });
    
    // Main task to handle incoming messages
    while let Some(message) = rx.recv().await {
        match message {
            MessageType::Error(err) => {
                    error!("Error from server: {}", err);
            }
            MessageType::Text(text) => {
                info!("Server response: {}",text);
            }
            _ => {
             info!("Received unexpected message from server");   
            }
        }
    }
    
    Ok(())
}
    
    

// Function for handling user input
async fn handle_user_input(writer: Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>) -> Result<()> {
    let stdin = io::stdin();
    let mut stdin_reader = BufReader::new(stdin).lines();
    
    // List of valid commands
    let valid_commands = [".file", ".image", ".quit"];
              
    while let Some(line) = stdin_reader.next_line().await? {
        let input = line.trim().to_string();
        info!("Read input: {}",input);
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
                match File::open(&path).await {
                    Ok(mut file) => { 
                        let mut buffer = Vec::new();
                        file.read_to_end(&mut buffer).await.context("Failed to read file")?;
                        let filename = std::path::Path::new(path)
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string();
                        let message = MessageType::File(filename.clone(), buffer);
                        send_message(&writer, &message).await?;
                        info!("Sent file: {}", filename);
                    }
                    Err(e) => {
                       eprintln!("Failed to open file {}: {}", path, e);
                    }
                }
           }
           ".image" => {
               if input.len() <= 7 {
                   eprintln!("Error: .image command requires a file path.");
                   continue;
               }
               let path = &input[7..].trim();
               match File::open(&path).await {
                 Ok(mut file) => { 
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer).await.context("Failed to read image")?;
                    let message = MessageType::Image(buffer);
                    send_message(&writer, &message).await?;
                    info!("Sent image from path: {}", path);
                 }
                 Err(e) => {
                    eprintln!("Failed to open image {}: {}", path, e);
                 }
               }
           }
           ".quit" => {
               let message = MessageType::Quit;
               send_message(&writer, &message).await?;
               info!("Sent quit message");
               break;
           }
           _ => {}
               }  
        } else {
            let message = MessageType::Text(input.to_string());
            send_message(&writer, &message).await?;
            info!("Sent text message: {}", input);
        }
    }
   
    Ok(())    

}

async fn handle_server_response(
    reader: Arc<Mutex<tokio::net::tcp::OwnedReadHalf>>,
    tx: mpsc::Sender<MessageType>
) -> Result<()> {
    loop {
        let mut len_bytes = [0u8;4];
        {
            let mut reader = reader.lock().await;
            if let Err(e) = reader.read_exact(&mut len_bytes).await {
                error!("Failed to read response length: {}",e);
                return Err(e.into());
            }
        }
        let len = u32::from_be_bytes(len_bytes) as usize;
        
        let mut buffer = vec![0u8; len];
        {
            let mut reader = reader.lock().await;
            if let Err(e) = reader.read_exact(&mut buffer).await {
               error!("Failed to read response: {}", e);
               return Err(e.into());
            }
        }
        
        let message = deserialize_message(&buffer).map_err(|e| anyhow::anyhow!(e))?;
        info!("Received message from server: {:?}",message);
        if let Err(e) = tx.send(message).await {
           error!("Failed to send message to main loop: {}",e);
           return Err(e.into());
        }
      }
}



// Function to send a message to the server
async fn send_message(
    writer: &Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>,
    message: &MessageType,
) -> Result<()> {
    let serialized = serialize_message(message).map_err(|e| anyhow::anyhow!(e))?;

    // Send the length of the serialized message (as 4-byte value).
    let len = serialized.len() as u32;
    {
        let mut writer = writer.lock().await;
        writer.write_all(&len.to_be_bytes()).await.context("Failed to send message length")?;
        // Send the serialized message.
        writer.write_all(&serialized).await.context("Failed to send message")?;
    }
    
    info!("Sent message: {:?}", message); // T
    Ok(())
}

     
