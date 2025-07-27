use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt, copy_bidirectional};
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

    pub fn with_config(host: String, port: u16, target: Option<String>, max_connections: usize) -> Self {
        Self { 
            host, 
            port, 
            target,
            max_connections,
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
        
        // Handle CONNECT method for HTTPS tunneling
        if request.method.to_uppercase() == "CONNECT" {
            Self::handle_connect(client_socket, &request, timeout_duration).await?;
        } else {
            // Handle regular HTTP requests
            Self::handle_http(client_socket, client_addr, &request, &buffer[..n], target, timeout_duration).await?;
        }

        Ok(())
    }

    async fn handle_connect(
        mut client_socket: TcpStream,
        request: &http::Request,
        timeout_duration: Duration,
    ) -> Result<()> {
        // Parse the target from CONNECT request (format: host:port)
        let target_addr = &request.path;
        debug!("CONNECT request to {}", target_addr);

        // Connect to the target server
        let target_socket = timeout(timeout_duration, TcpStream::connect(target_addr)).await
            .map_err(|_| anyhow::anyhow!("Timeout connecting to target for CONNECT"))?
            .map_err(|e| anyhow::anyhow!("Failed to connect to target for CONNECT: {}", e))?;

        // Send 200 Connection Established response
        let response = b"HTTP/1.1 200 Connection Established\r\n\r\n";
        timeout(timeout_duration, client_socket.write_all(response)).await
            .map_err(|_| anyhow::anyhow!("Timeout writing CONNECT response"))?
            .map_err(|e| anyhow::anyhow!("Failed to write CONNECT response: {}", e))?;

        info!("CONNECT tunnel established to {}", target_addr);

        // Start bidirectional copying (tunnel mode)
        if let Err(e) = copy_bidirectional(&mut client_socket, &mut target_socket).await {
            debug!("Tunnel closed: {}", e);
        }

        info!("CONNECT tunnel closed for {}", target_addr);
        Ok(())
    }

    async fn handle_http(
        mut client_socket: TcpStream,
        client_addr: SocketAddr,
        _request: &http::Request,
        request_data: &[u8],
        target: Option<String>,
        timeout_duration: Duration,
    ) -> Result<()> {
        let target_host = target.unwrap_or_else(|| "httpbin.org:80".to_string());
        debug!("Connecting to target: {}", target_host);
        
        let mut target_socket = timeout(timeout_duration, TcpStream::connect(&target_host)).await
            .map_err(|_| anyhow::anyhow!("Timeout connecting to target"))?
            .map_err(|e| anyhow::anyhow!("Failed to connect to target {}: {}", target_host, e))?;
        
        // Forward the request
        timeout(timeout_duration, target_socket.write_all(request_data)).await
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