# Chat Application

This project is a simple chat application with seperate client and server parts. The shared functionality is abstracted into a shared library crate.

## Project Structure

- 'client/': The client application
- 'server/': The server application
- 'shared/': The shared library with common functionality

## How to Run

1. Clone the repository
2. Build the project
    'cargo build'
3. Run the server
    'cargo run --bin server -- 0.0.0.0:11111'
4. Run the client
    'cargo run --bin client --localhost:11111'
    
## Client Usage

The client can send different types of messages to the server. Here are the available commands:

- **Text Message**: Any text that does not start with a command will be sent as a text message.
    'Hello, this is a text message!'

- **Send Image**: Use the '.image <path>' command to send an image (assumed to be a '.png' file) to the server.
    '.image /path/to/your/image.png'
    
- **Send File**: Use the '.file <path>' command to send a file to the server.
    '.file /path/to/your/file.txt'

- **Quit**: Use the '.quit' command to disconnect the client from the server.
    '.quit'
    
## Dependencies

- 'serde'
- 'serde_cbor'
- 'chrono'
- 'log'
- 'tracing'
- 'tracing-subscriber'


## Comments and Documentation

The code is well-commented to explain the reassoning behind the implementation. Please refer to the comments in the source code for mode details.

The server will log received messages and store files and images in the 'files' and 'images' directories respectively.
