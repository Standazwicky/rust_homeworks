mod client;
mod message;
mod server;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <server|client> [address]", args[0]);
        return;
    }

    let mode = &args[1];
    let address = if args.len() > 2 {
        &args[2]
    } else {
        "localhost:11111"
    };

    if mode == "server" {
        server::listen_and_accept(address).unwrap();
    } else if mode == "client" {
        client::start_client(address).unwrap();
    } else {
        eprintln!("Unknown mode: {}", mode);
    }
}
