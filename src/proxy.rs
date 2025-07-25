use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{timeout, Duration};
use tracing::{info, error, debug, warn};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Semaphore;
use anyhow::Result;

use crate::http;

pub struct Proxy {
    host: String,
    port: u16,
    target: Option<String>,
    max_connections: usize,
    timeout: Duration,
}

impl Proxy {
    pub fn new(host: String, port: u16, target: Option<String>) -> Self {
        Self { 
            host, 
            port, 
            target,
            max_connections: 100,
            timeout: Duration::from_secs(30),
        }
    }

    pub async fn run(&self) -> Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr).await?;
        info!("Proxy listening on {} (max connections: {})", addr, self.max_connections);

        let semaphore = Arc::new(Semaphore::new(self.max_connections));

        loop {
            let (socket, client_addr) = listener.accept().await?;
            debug!("New connection from {}", client_addr);
            
            let permit = match semaphore.clone().try_acquire_owned() {
                Ok(permit) => permit,
                Err(_) => {
                    warn!("Connection limit reached, rejecting {}", client_addr);
                    drop(socket);
                    continue;
                }
            };
            
            let target = self.target.clone();
            let timeout_duration = self.timeout;
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(socket, client_addr, target, timeout_duration).await {
                    error!("Connection error for {}: {}", client_addr, e);
                }
                drop(permit);
            });
        }
    }

    async fn handle_connection(
        mut client_socket: TcpStream, 
        client_addr: SocketAddr, 
        target: Option<String>,
        timeout_duration: Duration,
    ) -> Result<()> {
        let mut buffer = [0; 1024];
        
        let n = timeout(timeout_duration, client_socket.read(&mut buffer)).await
            .map_err(|_| anyhow::anyhow!("Timeout reading from client"))?
            .map_err(|e| anyhow::anyhow!("Failed to read from client: {}", e))?;
        
        if n == 0 {
            debug!("Empty request from {}", client_addr);
            return Ok(());
        }

        let request = http::Request::parse(&buffer[..n])
            .map_err(|e| anyhow::anyhow!("Failed to parse request: {}", e))?;
        
        info!("{} {} {} from {}", request.method, request.path, request.version, client_addr);
        
        let target_host = target.unwrap_or_else(|| "httpbin.org:80".to_string());
        debug!("Connecting to target: {}", target_host);
        
        let mut target_socket = timeout(timeout_duration, TcpStream::connect(&target_host)).await
            .map_err(|_| anyhow::anyhow!("Timeout connecting to target"))?
            .map_err(|e| anyhow::anyhow!("Failed to connect to target {}: {}", target_host, e))?;
        
        // Forward the request
        timeout(timeout_duration, target_socket.write_all(&buffer[..n])).await
            .map_err(|_| anyhow::anyhow!("Timeout writing to target"))?
            .map_err(|e| anyhow::anyhow!("Failed to write to target: {}", e))?;

        // Read response and forward back
        let mut response_buffer = [0; 4096];
        let response_size = timeout(timeout_duration, target_socket.read(&mut response_buffer)).await
            .map_err(|_| anyhow::anyhow!("Timeout reading from target"))?
            .map_err(|e| anyhow::anyhow!("Failed to read response from target: {}", e))?;
            
        if response_size > 0 {
            timeout(timeout_duration, client_socket.write_all(&response_buffer[..response_size])).await
                .map_err(|_| anyhow::anyhow!("Timeout writing to client"))?
                .map_err(|e| anyhow::anyhow!("Failed to write response to client: {}", e))?;
            info!("Forwarded {} bytes response to {}", response_size, client_addr);
        } else {
            debug!("No response from target for {}", client_addr);
        }

        Ok(())
    }
} 