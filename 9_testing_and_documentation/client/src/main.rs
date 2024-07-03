use anyhow::{Context, Result};
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

/// Main function    
///
/// This function initializes the tracing subscriber for logging and parses
/// command-line arguments to determine the server address. It then calls
/// `start_client` to connect to server and handle client operations.
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

    if let Err(e) = start_client(address).await {
        error!("Error: {}", e);
    }
}

/// Starts the client and connects to the server
///
/// This function connects to the server at the specifies address, sets up
/// channels forcommunication, and spawns tasks to handle server responses
/// and user input.
///
/// # Arguments
///
/// * `addresess` - The server address to connect to.
async fn start_client(address: &str) -> Result<()> {
    let stream = TcpStream::connect(address)
        .await
        .context("Failed to connect to server")?;
    info!("Connected to server at {}", address);
    info!(
        "For login use: \n 
    .login <user> \n 
    For registration use: \n 
    .register <user> \n
    To exit the client use: \n
    .quit"
    );

    let (reader, writer) = stream.into_split();
    let reader = Arc::new(Mutex::new(reader));
    let writer = Arc::new(Mutex::new(writer));

    let (tx, mut rx) = mpsc::channel::<MessageType>(100);
    let (quit_tx, mut quit_rx) = mpsc::channel::<()>(1);

    // Task for handling server responses
    let reader_clone = Arc::clone(&reader);
    let tx_clone = tx.clone();
    let quit_tx_clone = quit_tx.clone();
    let server_response_handle = task::spawn(async move {
        if let Err(e) = handle_server_response(reader_clone, tx_clone, quit_tx_clone).await {
            error!("Error handling server response: {}", e);
        }
    });

    // Task for handling user input
    let writer_clone = Arc::clone(&writer);
    let user_input_handle = task::spawn(async move {
        if let Err(e) = handle_user_input(writer_clone, tx).await {
            error!("Error handling user input: {}", e);
        }
    });

    // Main task to handle incoming messages
    let incoming_handle = task::spawn(async move {
        while let Some(message) = rx.recv().await {
            match message {
                MessageType::Error(err) => {
                    error!("Error from server: {}", err);
                }
                MessageType::Text(text) => {
                    info!("Server response: {}", text);
                }
                MessageType::Quit => {
                    info!("Server has closed the connection.");
                    break;
                }
                _ => {
                    info!("Received unexpected message from server");
                }
            }
        }
        let _ = quit_tx.send(()).await; // signal to quit other tasks
    });

    tokio::select! {
        _ = incoming_handle => (),
        _ = user_input_handle => (),
        _ = server_response_handle => (),
        _ = quit_rx.recv() => {
         info!("Received quit signal, shutting down...");
        }
    }

    Ok(())
}

/// Handles user input
///
/// This function reads user input from command line, process commands
/// for sending text, files, images, and quit messages to the server.
///
/// # Arguments
///
/// * `writer` - A shared reference to the writer half of the TcpStream.
async fn handle_user_input(
    writer: Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>,
    tx: mpsc::Sender<MessageType>,
) -> Result<()> {
    let stdin = io::stdin();
    let mut stdin_reader = BufReader::new(stdin).lines();

    // List of valid commands
    let valid_commands = [".file", ".image", ".quit", ".login", ".register"];

    while let Some(line) = stdin_reader.next_line().await? {
        let input = line.trim().to_string();
        info!("Read input: {}", input);

        // Check if the input is command
        if input.starts_with('.') {
            let command = input.split_whitespace().next().unwrap_or("");

            // Check if command is valid
            if !valid_commands.contains(&command) {
                eprintln!("Invalid command. Valid commands are: .file <path>, .image <path>, .quit, .login <username>, .register <username>");
                continue;
            }

            match command {
                ".file" => {
                    if input.len() <= 6 {
                        eprintln!("Error: .file command requires a file path.");
                        continue;
                    }
                    let path = &input[6..].trim();
                    match File::open(&path).await {
                        Ok(mut file) => {
                            let mut buffer = Vec::new();
                            file.read_to_end(&mut buffer)
                                .await
                                .context("Failed to read file")?;
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
                            file.read_to_end(&mut buffer)
                                .await
                                .context("Failed to read image")?;
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
                    if let Err(e) = tx.send(message).await {
                        error!("Failed to send quit message to main loop: {}", e)
                    }
                    break;
                }
                ".login" => {
                    if input.len() <= 7 {
                        eprintln!("Error: .login command requires a username.");
                        continue;
                    }
                    let username = &input[7..].trim().to_string();
                    let message = MessageType::Login(username.clone());
                    send_message(&writer, &message).await?;
                    info!("Sent login message for username: {}", username);
                }
                ".register" => {
                    if input.len() <= 10 {
                        eprintln!("Error: .register command requires a username.");
                        continue;
                    }
                    let username = &input[10..].trim().to_string();
                    let message = MessageType::Register(username.clone());
                    send_message(&writer, &message).await?;
                    info!("Sent register message for username: {}", username);
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

/// Handles server responses
/// This function reads responses from the server, deserializes the messages,
/// and sends them to the main task through the provided channel.
///
/// # Arguments
///
/// * `reader` - A shared reference to the reader half of the TcpStream
/// * `tx` - A channel sender for sending messages to the main task.
async fn handle_server_response(
    reader: Arc<Mutex<tokio::net::tcp::OwnedReadHalf>>,
    tx: mpsc::Sender<MessageType>,
    quit_tx: mpsc::Sender<()>,
) -> Result<()> {
    loop {
        let mut len_bytes = [0u8; 4];
        {
            let mut reader = reader.lock().await;
            if let Err(e) = reader.read_exact(&mut len_bytes).await {
                error!("Failed to read response length: {}", e);
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
        //  info!("Received message from server: {:?}",message);
        if let MessageType::Quit = message {
            if let Err(e) = tx.send(message).await {
                error!("Failed to send quit message to main loop: {}", e);
            }
            let _ = quit_tx.send(()).await;
            break;
        }
        if let Err(e) = tx.send(message).await {
            error!("Failed to send message to main loop: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

/// Sends a message to the server
///
/// This function serializes a message and sends it to the server through
/// the provided writer.
/// # Arguments
///
/// * `writer` - A shared reference to the writer half of the TcpStream.
/// * `message` - The message to be sent.
async fn send_message(
    writer: &Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>,
    message: &MessageType,
) -> Result<()> {
    let serialized = serialize_message(message).map_err(|e| anyhow::anyhow!(e))?;

    // Send the length of the serialized message (as 4-byte value).
    let len = serialized.len() as u32;
    {
        let mut writer = writer.lock().await;
        writer
            .write_all(&len.to_be_bytes())
            .await
            .context("Failed to send message length")?;
        // Send the serialized message.
        writer
            .write_all(&serialized)
            .await
            .context("Failed to send message")?;
    }

    info!("Sent message: {:?}", message); // T
    Ok(())
}
