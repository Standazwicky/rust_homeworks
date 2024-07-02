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

type Clients = Arc<Mutex<HashMap<std::net::SocketAddr, (Arc<Mutex<TcpStream>>, String)>>>;

async fn handle_client(
    stream: Arc<Mutex<TcpStream>>,
    addr: std::net::SocketAddr,
    clients: Clients,
    db_pool: Pool<Postgres>,
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

async fn handle_message(
    addr: std::net::SocketAddr,
    message: MessageType,
    clients: Clients,
    db_pool: Pool<Postgres>,
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

async fn user_exists(db_pool: &Pool<Postgres>, username: &str) -> Result<bool> {
    let result = sqlx::query!("SELECT id FROM users WHERE username = $1", username)
        .fetch_optional(db_pool)
        .await?;

    Ok(result.is_some())
}

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

async fn fetch_messages(db_pool: &Pool<Postgres>) -> Result<Vec<(String, String, NaiveDateTime)>> {
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

async fn listen_and_accept(address: &str, db_pool: Pool<Postgres>) -> std::io::Result<()> {
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

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
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

    // Connect to database
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .context("Failed to connect to the database")?;

    // Test: Load messages
    let messages = fetch_messages(&db_pool).await?;
    for (username, content, timestamp) in messages {
        println!("{} [{}]: {}", username, timestamp, content);
    }

    if let Err(e) = listen_and_accept(address, db_pool).await {
        error!("Error: {}", e);
    }

    Ok(())
}
