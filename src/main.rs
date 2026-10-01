mod store;

use store::Store;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[derive(Debug)]
enum Command {
    Ping,
    Get(String),
    Set(String, String),
    Del(String),
}

fn parse_command(data: &str) -> Option<Command> {
    let parts: Vec<&str> = data.split("\r\n").collect();

    let count: usize = parts.first()?.strip_prefix('*')?.parse().ok()?;

    let mut args = Vec::new();
    let mut index = 1;

    for _ in 0..count {
        let length: usize = parts.get(index)?.strip_prefix('$')?.parse().ok()?;

        let value = *parts.get(index + 1)?;

        if value.len() != length {
            return None;
        }

        args.push(value);
        index += 2;
    }

    match args.as_slice() {
        [command] if command.eq_ignore_ascii_case("PING") => {
            Some(Command::Ping)
        }

        [command, key] if command.eq_ignore_ascii_case("GET") => {
            Some(Command::Get((*key).to_string()))
        }

        [command, key, value] if command.eq_ignore_ascii_case("SET") => {
            Some(Command::Set(
                (*key).to_string(),
                (*value).to_string(),
            ))
        }

        [command, key] if command.eq_ignore_ascii_case("DEL") => {
            Some(Command::Del((*key).to_string()))
        }

        _ => None,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    println!("Rust_EDIS server running on 127.0.0.1:6379...");

    let store = Store::new();

    loop {
        let (mut socket, addr) = listener.accept().await?;

        println!("New client connected: {}", addr);

        let store = store.clone();

        tokio::spawn(async move {
            let mut buf = [0; 512];

            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => return,

                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buf[..n]);

                        println!("Received: {:?}", data);

                        if let Some(command) = parse_command(&data) {
                            println!("Command: {:?}", command);

                            match command {
                                Command::Ping => {
                                    if let Err(e) =
                                        socket.write_all(b"+PONG\r\n").await
                                    {
                                        eprintln!(
                                            "Failed to write to socket: {}",
                                            e
                                        );
                                        return;
                                    }
                                }

                                Command::Set(key, value) => {
                                    store.set(key, value);

                                    if let Err(e) =
                                        socket.write_all(b"+OK\r\n").await
                                    {
                                        eprintln!(
                                            "Failed to write to socket: {}",
                                            e
                                        );
                                        return;
                                    }
                                }

                                Command::Get(key) => {
                                    match store.get(&key) {
                                        Some(value) => {
                                            let response =
                                                format!("${}\r\n{}\r\n", value.len(), value);

                                            if let Err(e) =
                                                socket.write_all(response.as_bytes()).await
                                            {
                                                eprintln!(
                                                    "Failed to write to socket: {}",
                                                    e
                                                );
                                                return;
                                            }
                                        }

                                        None => {
                                            if let Err(e) =
                                                socket.write_all(b"$-1\r\n").await
                                            {
                                                eprintln!(
                                                    "Failed to write to socket: {}",
                                                    e
                                                );
                                                return;
                                            }
                                        }
                                    }
                                }

                                Command::Del(key) => {
                                    let removed = store.del(&key);

                                    let response = if removed {
                                        b":1\r\n"
                                    } else {
                                        b":0\r\n"
                                    };

                                    if let Err(e) =
                                        socket.write_all(response).await
                                    {
                                        eprintln!(
                                            "Failed to write to socket: {}",
                                            e
                                        );
                                        return;
                                    }
                                }
                            }
                        }
                    }

                    Err(e) => {
                        eprintln!("Failed to read from socket: {}", e);
                        return;
                    }
                }
            }
        });
    }
}
