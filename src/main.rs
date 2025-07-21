use clap::Parser;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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

    #[arg(short, long)]
    target: Option<String>,
}

async fn handle_connection(mut client_socket: TcpStream, target: Option<String>) {
    let mut buffer = [0; 1024];
    match client_socket.read(&mut buffer).await {
        Ok(n) if n > 0 => {
            match http::Request::parse(&buffer[..n]) {
                Ok(request) => {
                    println!("Parsed request: {} {}", request.method, request.path);
                    
                    // Simple forwarding to httpbin.org for testing
                    let target_host = target.unwrap_or_else(|| "httpbin.org:80".to_string());
                    
                    match TcpStream::connect(&target_host).await {
                        Ok(mut target_socket) => {
                            // Forward the request
                            if let Err(e) = target_socket.write_all(&buffer[..n]).await {
                                println!("Failed to write to target: {}", e);
                                return;
                            }

                            // Read response and forward back
                            let mut response_buffer = [0; 4096];
                            match target_socket.read(&mut response_buffer).await {
                                Ok(response_size) if response_size > 0 => {
                                    if let Err(e) = client_socket.write_all(&response_buffer[..response_size]).await {
                                        println!("Failed to write response to client: {}", e);
                                    } else {
                                        println!("Forwarded {} bytes response to client", response_size);
                                    }
                                }
                                Ok(_) => println!("No response from target"),
                                Err(e) => println!("Failed to read response from target: {}", e),
                            }
                        }
                        Err(e) => {
                            println!("Failed to connect to target {}: {}", target_host, e);
                        }
                    }
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!("Starting Tachyon Proxy on {}:{}", args.host, args.port);
    if let Some(ref target) = args.target {
        println!("Default target: {}", target);
    }
    
    let addr = format!("{}:{}", args.host, args.port);
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on {}", addr);

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {}", addr);
        
        let target = args.target.clone();
        tokio::spawn(async move {
            handle_connection(socket, target).await;
        });
    }
} 