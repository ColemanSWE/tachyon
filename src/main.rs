use clap::Parser;
use tokio::net::TcpListener;
use std::io;

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
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {}", addr);
        
        tokio::spawn(async move {
            // TODO: Handle connection
            println!("Handling connection from {}", addr);
        });
    }
} 