use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// Implementing a basic tcp server
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    println!("Rust_EDIS server running on 127.0.0.1:6379...");

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("New client connected: {}", addr);

        tokio::spawn(async move {
            let mut buf = [0; 512];

            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(0) => return, // Connection closed by client
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("Failed to read from socket: {}", e);
                        return;
                    }
                };

                // Echo back a basic response for testing
                if let Err(e) = socket.write_all(b"+PONG\r\n").await {
                    eprintln!("Failed to write to socket: {}", e);
                    return;
                }
            }
        });
    }
}
