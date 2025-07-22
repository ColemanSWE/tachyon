use clap::Parser;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, error, debug};
use std::net::SocketAddr;

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

    #[arg(short, long, default_value = "info")]
    log_level: String,
}

async fn handle_connection(mut client_socket: TcpStream, client_addr: SocketAddr, target: Option<String>) -> anyhow::Result<()> {
    let mut buffer = [0; 1024];
    let n = client_socket.read(&mut buffer).await?;
    
    if n == 0 {
        debug!("Empty request from {}", client_addr);
        return Ok(());
    }

    let request = http::Request::parse(&buffer[..n])
        .map_err(|e| anyhow::anyhow!("Failed to parse request: {}", e))?;
    
    info!("{} {} {} from {}", request.method, request.path, request.version, client_addr);
    
    // Simple forwarding to httpbin.org for testing
    let target_host = target.unwrap_or_else(|| "httpbin.org:80".to_string());
    debug!("Connecting to target: {}", target_host);
    
    let mut target_socket = TcpStream::connect(&target_host).await
        .map_err(|e| anyhow::anyhow!("Failed to connect to target {}: {}", target_host, e))?;
    
    // Forward the request
    target_socket.write_all(&buffer[..n]).await
        .map_err(|e| anyhow::anyhow!("Failed to write to target: {}", e))?;

    // Read response and forward back
    let mut response_buffer = [0; 4096];
    let response_size = target_socket.read(&mut response_buffer).await
        .map_err(|e| anyhow::anyhow!("Failed to read response from target: {}", e))?;
        
    if response_size > 0 {
        client_socket.write_all(&response_buffer[..response_size]).await
            .map_err(|e| anyhow::anyhow!("Failed to write response to client: {}", e))?;
        info!("Forwarded {} bytes response to {}", response_size, client_addr);
    } else {
        debug!("No response from target for {}", client_addr);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(format!("tachyon_proxy={}", args.log_level))
        .init();

    info!("Starting Tachyon Proxy on {}:{}", args.host, args.port);
    if let Some(ref target) = args.target {
        info!("Default target: {}", target);
    }
    
    let addr = format!("{}:{}", args.host, args.port);
    let listener = TcpListener::bind(&addr).await?;
    info!("Proxy listening on {}", addr);

    loop {
        let (socket, client_addr) = listener.accept().await?;
        debug!("New connection from {}", client_addr);
        
        let target = args.target.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, client_addr, target).await {
                error!("Connection error for {}: {}", client_addr, e);
            }
        });
    }
} 