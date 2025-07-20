use clap::Parser;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt};
use std::io;

mod http;

#[derive(Parser, Debug)]
#[command(name = "tachyon-proxy")]
#[command(about = "A high-performance HTTP proxy written in Rust")]
struct Args {
    #[arg(short, long, default_value = "8080")]
    port: u16,

    #[arg(short, long, default_value = "127.0.0.1")]
    host: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!("Starting Tachyon Proxy on {}:{}", args.host, args.port);
    
    let addr = format!("{}:{}", args.host, args.port);
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on {}", addr);

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("New connection from {}", addr);
        
        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            match socket.read(&mut buffer).await {
                Ok(n) if n > 0 => {
                    match http::Request::parse(&buffer[..n]) {
                        Ok(request) => {
                            println!("Parsed request: {} {}", request.method, request.path);
                        }
                        Err(e) => {
                            println!("Failed to parse request: {}", e);
                        }
                    }
                }
                Ok(_) => {
                    println!("Connection closed by client");
                }
                Err(e) => {
                    println!("Failed to read from socket: {}", e);
                }
            }
        });
    }
} 