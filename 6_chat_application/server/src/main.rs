use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread;
use std::fs;
use std::path::Path;
use std::env;
use chrono::Utc;
use shared::{MessageType, deserialize_message}; 



pub fn handle_client(mut stream: TcpStream) -> Result<MessageType, Box<dyn std::error::Error>> {
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes)?;
    let len = u32::from_be_bytes(len_bytes) as usize;

    let mut buffer = vec![0u8; len];
    stream.read_exact(&mut buffer)?;

    let message = deserialize_message(&buffer)?;
    Ok(message)
}

pub fn handle_message(
    addr: SocketAddr,
    message: MessageType,
) -> Result<bool, Box<dyn std::error::Error>> {
    match message {
        MessageType::Quit => {
            println!("Client {} sent quit message", addr);
            return Ok(true);
        }
        MessageType::Text(text) => {
            println!("Text message from {}: {}", addr, text);
        }
        MessageType::Image(data) => {
            ensure_directories_exist()?;
            println!("Receiving image from {}...", addr);
            let timestamp = Utc::now().timestamp();
            let filename = format!("images/{}.png", timestamp);
            fs::write(&filename, &data)?;
            println!("Saved image to {}", filename);
        }
        MessageType::File(name, data) => {
            ensure_directories_exist()?;
            println!("Receiving file '{}' from {}...", name, addr);
            let filename = format!("files/{}", name);
            fs::write(&filename, &data)?;
            println!("Saved file to {}", filename);
        }
    }
    Ok(false)
}

pub fn listen_and_accept(address: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(address)?;
    println!("Server running on {}", address);

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
                                println!("Error handling message from {}: {}", addr, e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error handling client {}: {}", addr, e);
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

fn ensure_directories_exist() -> Result<(), Box<dyn std::error::Error>> {
    let paths = ["images", "files"];
    for path in &paths {
        if !Path::new(path).exists() {
            fs::create_dir(path)?;
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
    
    if let Err(e) = listen_and_accept(address) {
     eprintln!("Error: {}",e);   
    }   
}
