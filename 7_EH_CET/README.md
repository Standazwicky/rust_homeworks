# Chat Application

## Overview

This project implements a chat application with a client-server architecture. The server can handle multiple clients, receive text messages, files and images, convert images to PNG format, and save them. The application includes robust error handling using the `anyhow` and `thiserror` crates.

## Features

- Clients cand send text messages, files, and images.
- The server converts received images to PNG format before saving them.
- Robust error handling and logging using `anyhow` and `thiserror`.
- Clients receive acknowladgments and error messages from the server.

## Dependencies

- `anyhow`
- `thiserror`
- `image`
- `chrono`
- `tracing`
- `tracing-subscriber`
- `serde`
- `serde_cbor`

## Project Structure

- `client/`: The client application
- `server/`: The server application
- `shared/`: The shared library with common functionality

## How to Run

1. Clone the repository
2. Build the project
    ```sh 
    cargo build
    ```
3. Run the server, optionally specifying the address and port (default  is `localhost:11111`):
    ```sh
    cargo run --bin server -- 0.0.0.0:11111
    ```
4. Run the client, specifying the server address and port (default is `localhost:11111`):
    ```sh
    cargo run --bin client --localhost:11111
    ```
    
## Client Usage

The client can send different types of messages to the server. Here are the available commands:

- **Text Message**: Any text that does not start with a command will be sent as a text message.
    ```sh 
    Hello, this is a text message!
    ```

- **Send Image**: Use the `.image <path>` to send an image (will be converted to PNG).
    ```sh
    .image /path/to/your/image.png
    ```
    
- **Send File**: Use the `.file <path>` command to send a file to the server.
    ```sh
    .file /path/to/your/file.txt
    ```

- **Quit**: Use the `.quit` command to disconnect the client from the server and quit the client.
    ```sh
    .quit
    ```
    
## Example
1. Start the server:
    ```sh
    cargo run --bin server
    ```
2. Start the client:
    ```sh
    cargo run --bin client
    ```
3. Send a text message:
    ```sh
    Hello, Server!
    ```
4. Send a file:
    ```sh
    .file /path/to/your/file.txt
    ```
5. Send an image:
    ```sh
    .image /path/to/your/image.png
    ```
6. Quit the client:
    ```sh
    .quit
    ```

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.
