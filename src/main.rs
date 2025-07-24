use clap::Parser;
use tracing::{info, error};

mod http;
mod proxy;

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

    let proxy = proxy::Proxy::new(args.host, args.port, args.target);
    
    if let Err(e) = proxy.run().await {
        error!("Proxy error: {}", e);
        std::process::exit(1);
    }

    Ok(())
} 