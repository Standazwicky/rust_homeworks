# Asynchronous Chat Application with database Integration

## Overview

This project is an asynchronous chat application built using Rust, Tokio, and Actix-web, with a PostgreSQL database for storing user data and chat messages. The application supports user registration, login, and message exchange, including text, images, and files. The web server serves a static HTML file for the client interface.

## Features

- Clients cand send text messages, files, and images.
- Robust error handling and logging using `anyhow` and `thiserror`.
- Clients receive acknowladgments and error messages from the server.
- Asynchronous I/O operations using Tokio
- User registration and login
- Persistent storage of user data and messages in a PostgreSQL database
- Web server using Actix-web to serve static files.
- Easy setup and configuration using environment variables.

## Dependencies

- `anyhow`
- `thiserror`
- `image`
- `chrono`
- `tracing`
- `tracing-subscriber`
- `serde`
- `serde_cbor`
- `sqlx`
- `dotenv`
- `actix-web`
- `actix-files`

## Project Structure

- `client/`: The client application
- `server/`: The server application
- `shared/`: The shared library with common functionality

## Setup

### 1. Install Rust and Cargo

Follow the instructions on the [Rust website](https://www.rust-lang.org/tools/install) to install Rust and Cargo.

### 2. Install PostgreSQL

#### On Ubuntu:

```sh
sudo apt update
sudo apt install postgresql postgresql-contrib
```

#### On MacOS(using Homebrew):

```sh
 brew install postgresql
```

### 3. Set Up PostgreSQL

 Start the PostgreSQL service:
 
#### On Ubuntu:

```sh
 sudo service postgresql start
```

#### On MacOS:

```sh
 brew services start posgresql
```

Vreate a new PostgreSQL user and database:

```sh
 sudo -u postgres createuser chat_user
 sudo -u postgres ctreatedb chat_app
```

Set a password for the new PostgreSQL user:

```sh
 sudo -u postgres psql
 \password chat_user
```

Edit the `pg_hba.conf` file to use MD5 authentication:

```sh
# Open the pg_hba.conf file
sudo nano /etc/postgresql/12/main/pg_hba.conf

# Change this line
local   all             all                                     peer

# To this line
local   all             all                                     md5
```

Reload PostgreSQL configuration:

```sh
 sudo service postgresql restart
```

### 4.Create Tables

 Connect to the `chat_app` database and create the necessary tables:
 
 ```sh
  sudo -u postgres psql -d chat_app
 ```

 Create the `users` and `messages` tables:
 
 ```sql
  CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL
  );

  CREATE TABLE messages (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    content TEXT NOT NULL,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
  );
 ```
 
 ### 5.Environment Variables
 
 Create a `.env` file in the root of the project and add your database URL:
 
 ```dotenv
  DATABASE_URL=postgres:://chat_user:your_password@localhost/chat_app
 ```
 
 Replace `your_password` with the password you set for the `chat_user` PostgreSQL user.


## How to Run

1. Clone the repository
    ```sh
    git clone <repository_url>
    ```
2. Build the project
    ```sh 
    cargo build
    ```
3. Ensure the `static` directory exists in the build output:
    ```sh
    cp -u server/static target/debug/ 
    ```
4. Run the server, optionally specifying the address and port (default  is `localhost:11111`):
    ```sh
    cargo run --bin server -- 0.0.0.0:11111
    ```
5. Run the client, specifying the server address and port (default is `localhost:11111`):
    ```sh
    cargo run --bin client --localhost:11111
    ```

## Web Interface
The server includes a web interface that serves static files from the `static` directory. To access the web interface, open your web browser and navigate to:
    ```dotenv
    http://localhost:8080
    ```
    
Make sure the `static` directory contains an `index.html` file. 
 
## Client Usage

The client can send different types of messages to the server. Here are the available commands:

- **Register a New User:**Use the `.register <username>` command to register a new user.
```sh
 .register new_user
```

- **Login:** Use the `.login <username>` command to login with an existing user.
```sh
 .login existing_user
```

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
3. Register a new user:
    ```sh
    .register new_user
    ```
4. Login:
    ```sh
    .login existing_user
    ```
5. Send a text message:
    ```sh
    Hello, Server!
    ```
6. Send a file:
    ```sh
    .file /path/to/your/file.txt
    ```
7. Send an image:
    ```sh
    .image /path/to/your/image.png
    ```
8. Quit the client:
    ```sh
    .quit
    ```

## License

This project is licensed under the MIT License.
