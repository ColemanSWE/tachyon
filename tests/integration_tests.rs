use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;

#[tokio::test]
async fn test_proxy_basic_forwarding() {
    // Start a mock server
    let mock_server = start_mock_server().await;
    let mock_port = mock_server.local_addr().unwrap().port();
    
    // Start the proxy
    let proxy = tachyon_proxy::proxy::Proxy::new(
        "127.0.0.1".to_string(),
        0, // Let OS choose port
        Some(format!("http://127.0.0.1:{}", mock_port))
    );
    
    // Test basic HTTP request through proxy
    tokio::spawn(async move {
        proxy.run().await.unwrap();
    });
    
    // Give proxy time to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Make request through proxy
    let mut client = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    let request = b"GET /test HTTP/1.1\r\nHost: example.com\r\n\r\n";
    
    client.write_all(request).await.unwrap();
    
    let mut response = Vec::new();
    let mut buffer = [0; 1024];
    
    let n = timeout(Duration::from_secs(5), client.read(&mut buffer))
        .await
        .unwrap()
        .unwrap();
    
    response.extend_from_slice(&buffer[..n]);
    
    let response_str = String::from_utf8_lossy(&response);
    assert!(response_str.contains("HTTP/1.1 200 OK"));
    assert!(response_str.contains("Hello from mock server"));
}

#[tokio::test]
async fn test_proxy_timeout_handling() {
    // Test that proxy handles timeouts correctly
    let proxy = tachyon_proxy::proxy::Proxy::new(
        "127.0.0.1".to_string(),
        0,
        Some("http://192.0.2.1:80".to_string()) // Non-routable IP for timeout
    );
    
    tokio::spawn(async move {
        proxy.run().await.unwrap();
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let mut client = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    let request = b"GET /test HTTP/1.1\r\nHost: example.com\r\n\r\n";
    
    client.write_all(request).await.unwrap();
    
    // Should timeout and close connection
    let mut buffer = [0; 1024];
    let result = timeout(Duration::from_secs(35), client.read(&mut buffer)).await;
    
    // Connection should be closed due to timeout
    assert!(result.is_err() || result.unwrap().unwrap() == 0);
}

#[tokio::test]
async fn test_proxy_concurrent_connections() {
    let mock_server = start_mock_server().await;
    let mock_port = mock_server.local_addr().unwrap().port();
    
    let proxy = tachyon_proxy::proxy::Proxy::new(
        "127.0.0.1".to_string(),
        0,
        Some(format!("http://127.0.0.1:{}", mock_port))
    );
    
    tokio::spawn(async move {
        proxy.run().await.unwrap();
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Make multiple concurrent requests
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let handle = tokio::spawn(async move {
            let mut client = TcpStream::connect("127.0.0.1:8080").await.unwrap();
            let request = format!("GET /test{} HTTP/1.1\r\nHost: example.com\r\n\r\n", i);
            
            client.write_all(request.as_bytes()).await.unwrap();
            
            let mut response = Vec::new();
            let mut buffer = [0; 1024];
            
            let n = timeout(Duration::from_secs(5), client.read(&mut buffer))
                .await
                .unwrap()
                .unwrap();
            
            response.extend_from_slice(&buffer[..n]);
            String::from_utf8_lossy(&response).to_string()
        });
        
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    for handle in handles {
        let response = handle.await.unwrap();
        assert!(response.contains("HTTP/1.1 200 OK"));
    }
}

async fn start_mock_server() -> TcpListener {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    
    tokio::spawn(async move {
        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            
            tokio::spawn(async move {
                let mut buffer = [0; 1024];
                let _ = socket.read(&mut buffer).await;
                
                let response = b"HTTP/1.1 200 OK\r\nContent-Length: 21\r\n\r\nHello from mock server";
                let _ = socket.write_all(response).await;
            });
        }
    });
    
    listener
}

#[cfg(test)]
mod proxy_tests {
    use super::*;
    use tachyon_proxy::proxy::Proxy;

    #[test]
    fn test_parse_target_url() {
        // Test HTTP URLs
        let (host, port) = Proxy::parse_target_url("http://example.com").unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(port, 80);
        
        let (host, port) = Proxy::parse_target_url("http://example.com:8080").unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(port, 8080);
        
        // Test HTTPS URLs
        let (host, port) = Proxy::parse_target_url("https://example.com").unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(port, 443);
        
        let (host, port) = Proxy::parse_target_url("https://example.com:9443").unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(port, 9443);
        
        // Test host:port format
        let (host, port) = Proxy::parse_target_url("example.com:3000").unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(port, 3000);
        
        // Test just hostname
        let (host, port) = Proxy::parse_target_url("example.com").unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(port, 80);
    }
} 