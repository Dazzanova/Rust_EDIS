use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

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
                match socket.read(&mut buf).await {
                    Ok(0) => return,

                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buf[..n]);
                        println!("Received: {:?}", data);
                    }

                    Err(e) => {
                        eprintln!("Failed to read from socket: {}", e);
                        return;
                    }
                }

                if let Err(e) = socket.write_all(b"+PONG\r\n").await {
                    eprintln!("Failed to write to socket: {}", e);
                    return;
                }
            }
        });
    }
}
