use clap::Parser;
use tracing::{info, error};

mod http;
mod proxy;
mod config;

#[derive(Parser, Debug)]
#[command(name = "tachyon")]
#[command(about = "A high-performance HTTP proxy written in Rust")]
struct Args {
    #[arg(short, long, default_value = "8080")]
    port: u16,

    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    #[arg(short, long)]
    target: Option<String>,

    #[arg(short, long, default_value = "info")]
    log_level: String,

    #[arg(long)]
    config: Option<String>,

    #[arg(long, default_value = "100")]
    max_connections: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(format!("tachyon_proxy={}", args.log_level))
        .init();

    // Load configuration
    let config = if let Some(config_path) = args.config {
        info!("Loading configuration from {}", config_path);
        config::ProxyConfig::from_file(&config_path)?
    } else {
        config::ProxyConfig::default()
    };

    // Override config with CLI arguments
    let host = args.host;
    let port = args.port;
    let target = args.target.or(config.upstream.and_then(|u| u.default_target));
    let max_connections = args.max_connections;

    info!("Starting Tachyon Proxy on {}:{}", host, port);
    if let Some(ref target) = target {
        info!("Default target: {}", target);
    }
    info!("Max connections: {}", max_connections);

    let proxy = proxy::Proxy::with_config(host, port, target, max_connections);
    
    if let Err(e) = proxy.run().await {
        error!("Proxy error: {}", e);
        std::process::exit(1);
    }

    Ok(())
} 