use anyhow::{Context, Result};
use chrono::{NaiveDateTime, Utc};
use dotenv::dotenv;
use image::ImageFormat;
use shared::{deserialize_message, serialize_message, MessageType};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::task;
use tracing::{error, info};
use tracing_subscriber;
use actix_rt;

mod web_server; 

type Clients = Arc<Mutex<HashMap<std::net::SocketAddr, (Arc<Mutex<TcpStream>>, String)>>>;

/// Handles client connections and interactions
///
/// This functions manages the client's connection lifecycle, including login,
/// registration, and message handling.
///
/// # Arguments
///
/// * `stream` - The client's TCP stream wrapped in an Arc and Mutex.
/// * `addr` - A shared reference to the clients hashmap.
/// * `db_pool` - The PostgreSQL connection pool.
async fn handle_client(
    stream: Arc<Mutex<TcpStream>>,
    addr: std::net::SocketAddr,
    clients: Clients,
    db_pool: Arc<Pool<Postgres>>,
) -> Result<()> {
    // Ask for login or registration
    loop {
        let message = read_message(stream.clone()).await?;
        match message {
            MessageType::Login(username) => {
                if user_exists(&db_pool, &username).await? {
                    clients
                        .lock()
                        .await
                        .insert(addr, (stream.clone(), username.clone()));
                    info!("User {} logged in from {}", username, addr);
                    let welcome_message = MessageType::Text(format!("Welcome, {}!", username));
                    send_message(stream.clone(), &welcome_message).await?;
                    break;
                } else {
                    let error_message = MessageType::Error(
                        "User does not exist. You can create new user by \n 
               .register <username> 
               \n or provide correct username and login by 
               \n .login <username>"
                            .to_string(),
                    );
                    send_message(stream.clone(), &error_message).await?;
                }
            }
            MessageType::Register(username) => {
                if register_user(&db_pool, &username).await.is_ok() {
                    clients
                        .lock()
                        .await
                        .insert(addr, (stream.clone(), username.clone()));
                    info!("User {} registered and logged in from {}", username, addr);
                    let welcome_message =
                        MessageType::Text(format!("User {} registered successfully", username));
                    send_message(stream.clone(), &welcome_message).await?;
                    break;
                } else {
                    let error_message = MessageType::Error("Failed to register user.".to_string());
                    send_message(stream.clone(), &error_message).await?;
                }
            }
            MessageType::Quit => {
                info!("Client {} disconnected before login", addr);
                return Ok(());
            }
            _ => {
                let error_message = MessageType::Error(
                    "Please login or register. \n .login <username> \n or \n .register <username>"
                        .to_string(),
                );
                send_message(stream.clone(), &error_message).await?;
            }
        }
    }

    loop {
        match read_message(stream.clone()).await {
            Ok(message) => {
                match message {
                    MessageType::Login(username) => {
                        clients
                            .lock()
                            .await
                            .insert(addr, (stream.clone(), username.clone()));
                        info!("User {} connected from {}", username, addr);
                        // Send a welcome message or confirmation
                        let welcome_message = MessageType::Text(format!("Welcome, {}!", username));
                        send_message(stream.clone(), &welcome_message).await?;
                    }
                    _ => {
                        if handle_message(addr, message, clients.clone(), db_pool.clone()).await? {
                            break; // .quit message
                        }
                    }
                }
            }
            Err(e) => {
                error!("Error handling client {}: {:?}", addr, e);
                report_error(stream.clone(), &e.to_string()).await?;
                break;
            }
        }
    }

    clients.lock().await.remove(&addr);
    Ok(())
}

/// Reads a message from the client
///
/// This function reads a message from the clitnt's stream, deserializes it,
/// and returns the deserialized message.
///
/// # Arguments
///
/// * `stream` - The client's TCP stream wrapped in an Arc and Mutex.
async fn read_message(stream: Arc<Mutex<TcpStream>>) -> Result<MessageType> {
    let mut len_bytes = [0u8; 4];
    {
        let mut stream = stream.lock().await;
        stream
            .read_exact(&mut len_bytes)
            .await
            .context("Failed to read message lenght")?;
        //  info!("Received message length: {}", u32::from_be_bytes(len_bytes));
    }
    let len = u32::from_be_bytes(len_bytes) as usize;

    let mut buffer = vec![0u8; len];
    {
        let mut stream = stream.lock().await;
        stream
            .read_exact(&mut buffer)
            .await
            .context("Failed to read message")?;
        // info!("Received message: {:?}", buffer);
    }

    deserialize_message(&buffer).map_err(|e| anyhow::anyhow!(e))
}

/// Sends a message to the client
///
/// This function serializes a message and sends it to the client through
/// the provided stream.
///
/// # Arguments
/// * `stream` - The client's TCP stream wrapped in an Arc and Mutex.
/// * `message` - The message to be sent.
async fn send_message(stream: Arc<Mutex<TcpStream>>, message: &MessageType) -> Result<()> {
    let serialized = serialize_message(message).map_err(|e| anyhow::anyhow!(e))?;
    let len = serialized.len() as u32;
    {
        let mut stream = stream.lock().await;
        stream
            .write_all(&len.to_be_bytes())
            .await
            .context("Failed to send message length")?;
        stream
            .write_all(&serialized)
            .await
            .context("Failed to send message")?;
    }
    Ok(())
}

/// Handles messages from the client
/// This function processes messages from the client, including handling
/// text, image, and file messages, as well as the quit message.
///
/// # Arguments
///
/// * `addr` - The client's socket address.
/// * `message` - The message received from the client.
/// * `clients` - A shared reference to the clients hashmap.
/// * `db_pool` - The PostgreSQL connection pool.
async fn handle_message(
    addr: std::net::SocketAddr,
    message: MessageType,
    clients: Clients,
    db_pool: Arc<Pool<Postgres>>,
) -> Result<bool> {
    let username = {
        let clients = clients.lock().await;
        if let Some((_, username)) = clients.get(&addr) {
            username.clone()
        } else {
            return Err(anyhow::anyhow!("User not logged in"));
        }
    };

    match message {
        MessageType::Quit => {
            info!("User {} ({}) sent quit message", username, addr);
            clients.lock().await.remove(&addr);
            return Ok(true);
        }
        MessageType::Text(text) => {
            info!("Text message from {}: {}", username, text);
            save_message(&db_pool, username.to_string(), text).await?;
        }
        MessageType::Image(data) => {
            ensure_directories_exist().await?;
            info!("Receiving image from {}...", username);
            let timestamp = Utc::now().timestamp();
            let filename = format!("images/{}.png", timestamp);

            // Convert image to PNG format
            let data_clone = data.clone();
            let filename_clone = filename.clone();
            task::spawn_blocking(move || {
                let image = image::load_from_memory(&data_clone)
                    .context("Failed to load image from memory")?;
                let mut output_file = std::fs::File::create(&filename_clone)
                    .context("Failed to create image file")?;
                image
                    .write_to(&mut output_file, ImageFormat::Png)
                    .context("Failed to write image as PNG")?;
                Ok::<(), anyhow::Error>(())
            })
            .await??;

            info!("Saved image to {}", filename);
        }
        MessageType::File(name, data) => {
            ensure_directories_exist().await?;
            info!("Receiving file '{}' from {}...", name, addr);
            let filename = format!("files/{}", name);
            fs::write(&filename, &data)
                .await
                .context("Failed to save file")?;
            info!("Saved file to {}", filename);
        }
        MessageType::Error(err) => {
            error!("Error from {}: {}", addr, err);
        }
        MessageType::Login(_) => {
            error!("Received login message after user is already logged in");
        }
        MessageType::Register(_) => {
            error!("Received register message after user is already logged in")
        }
    }
    Ok(false)
}

/// Checks if a user exists in the database
///
/// This function queries the database to check if a user with the username exists.
///
/// # Arguments
///
/// * `db_pool` - The PostgreSQL connection pool.
/// # `username` - The username to check.
async fn user_exists(db_pool: &Pool<Postgres>, username: &str) -> Result<bool> {
    let result = sqlx::query!("SELECT id FROM users WHERE username = $1", username)
        .fetch_optional(db_pool)
        .await?;

    Ok(result.is_some())
}

/// Registers a new user in the database
///
/// This function inserts a new user with the specified username into the
/// database.
///
/// # Arguments
/// * `db_pool` - The PostgreSQL connection pool.
/// * `username` - The username to register.
async fn register_user(db_pool: &Pool<Postgres>, username: &str) -> Result<()> {
    let result = sqlx::query!("INSERT INTO users (username) VALUES ($1)", username)
        .execute(db_pool)
        .await;

    match result {
        Ok(_) => {
            info!("User {} registered successfully", username);
            Ok(())
        }
        Err(e) => {
            error!("Failed to register user {}: {:?}", username, e);
            Err(e.into())
        }
    }
}

/// Saves message to the database
///
/// This function saves a message to the database with the associated user ID.
///
/// # Arguments
/// * `db_pool` - The PostgreSQL connection pool.
/// * `username` - The username of the user sending message.
/// * `content` - The content of the message.
async fn save_message(db_pool: &Pool<Postgres>, username: String, content: String) -> Result<()> {
    // Find user_id according to username
    let user_id_result = sqlx::query!("SELECT id FROM users WHERE username = $1", username)
        .fetch_one(db_pool)
        .await;

    let user_id = match user_id_result {
        Ok(record) => record.id,
        Err(e) => {
            error!("Failed to find user_id for username {}: {:?}", username, e);
            return Err(e.into());
        }
    };

    // Convert user_id to &str
    //  let user_id_str = user_id.to_string();

    // Save message with correct user_id
    let result = sqlx::query!(
        "INSERT INTO messages (user_id, content) VALUES ($1, $2)",
        user_id,
        content
    )
    .execute(db_pool)
    .await;

    match result {
        Ok(_) => {
            info!("Message saved to database for user: {}", username);
            Ok(())
        }
        Err(e) => {
            error!("Failed to save message for user {}: {:?}", username, e);
            Err(e.into())
        }
    }
}

/// Fetches all messages from the database
///
/// This function retrieves all messages from the database and rerurns them as a vector.
///
/// # Arguments
///
/// * `db_pool` - The PostgreSQL connection pool.
async fn _fetch_messages(db_pool: &Pool<Postgres>) -> Result<Vec<(String, String, NaiveDateTime)>> {
    let rows = sqlx::query!(
        r#"
        SELECT users.username, messages.content, messages.timestamp
        FROM messages
        JOIN users ON messages.user_id = users.id
        "#
    )
    .fetch_all(db_pool)
    .await;

    match rows {
        Ok(rows) => {
            let messages = rows
                .into_iter()
                .map(|row| (row.username, row.content, row.timestamp.unwrap()))
                .collect::<Vec<_>>();
            info!("Fetch messages from database: {:?}", messages);
            Ok(messages)
        }
        Err(e) => {
            error!("Failed to fetch messages: {:?}", e);
            Err(e.into())
        }
    }
}

/// Reports an error to the client
///
/// This function sends an error message to the client.
///
/// # Arguments
/// * `stream` - The client's TCP stream wrapped in an Arc and Mutex.
/// # `error_message` - The error message to sent.
async fn report_error(stream: Arc<Mutex<TcpStream>>, error_message: &str) -> Result<()> {
    let error_message = MessageType::Error(error_message.to_string());
    let serialized = serialize_message(&error_message)?;
    let len = serialized.len() as u32;
    {
        let mut stream = stream.lock().await;
        stream
            .write_all(&len.to_be_bytes())
            .await
            .context("Failed to send error length")?;
        stream
            .write_all(&serialized)
            .await
            .context("Failed to send error message")?;
        info!("Sent error message: {:?}", error_message);
    }
    Ok(())
}

/// Listens for and accepts incoming connections
///
/// This function starts the server, listens for incoming connections,
/// and spawn tasks to handle each client.
///
/// # Arguments
///
/// * `address` - The address to bind the server to.
/// * `db_pool` - The PostgreSQL connection pool.
async fn listen_and_accept(address: &str, db_pool: Arc<Pool<Postgres>>) -> std::io::Result<()> {
    let listener = TcpListener::bind(address).await?;
    info!("Server running on {}", address);

    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (stream, addr) = listener.accept().await?;
        let stream = Arc::new(Mutex::new(stream));
        clients
            .lock()
            .await
            .insert(addr, (stream.clone(), String::new()));

        let clients = Arc::clone(&clients);
        let db_pool = db_pool.clone();
        task::spawn(async move {
            if let Err(e) = handle_client(stream, addr, clients, db_pool).await {
                error!("Error handling client {}: {:?}", addr, e);
            }
        });
    }
}

// Ensures necessary directories exist
///
/// This function checks if the directories for storing images and files exist,
/// and creates them if they do not.
async fn ensure_directories_exist() -> Result<()> {
    let paths = ["images", "files"];
    for path in &paths {
        if !Path::new(path).exists() {
            fs::create_dir(path)
                .await
                .context(format!("Failed to create directory {}", path))?;
        }
    }
    Ok(())
}

/// Main function
///
/// This functino initializes the tracing subscriber for logging, connects to
/// The PostgreSQL database, and starts the server.
#[actix_rt::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args: Vec<String> = env::args().collect();
    let address = if args.len() < 2 {
        println!("Usage: {} <address>", args[0]);
        println!("Setting default: localhost:11111");
        "localhost:11111".to_string()
    } else {
        args[1].clone()
    };

    // Connect to database
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = Arc::new(
        PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .context("Failed to connect to the database")?,
    );

    let tcp_server_address = address.clone();
    let tcp_server_db_pool = Arc::clone(&db_pool);
    let tcp_server = task::spawn(async move {
        if let Err(e) = listen_and_accept(&tcp_server_address, tcp_server_db_pool).await {
            error!("Error: {}", e);
        }    
    });
    
    let http_server_db_pool = Arc::clone(&db_pool);
    actix_rt::spawn(async move { 
        if let Err(e) = web_server::run(http_server_db_pool).await {
            error!("HTTP Server Error: {}", e);
        }
    });
    
    tcp_server.await?;

    Ok(())

    
}


