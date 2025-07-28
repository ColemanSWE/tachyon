# Tachyon Proxy

A high-performance HTTP proxy written in Rust, designed for learning systems programming concepts.

## Features

- **HTTP/1.1 Forward Proxy**: Routes HTTP traffic between clients and target servers
- **HTTPS CONNECT Tunneling**: Full support for HTTPS via HTTP CONNECT method
- **Async I/O**: Built with Tokio for high-performance concurrent connections
- **Connection Limiting**: Configurable maximum connection limits with semaphore-based throttling
- **Timeout Management**: Configurable timeouts for all network operations
- **Configuration System**: TOML-based configuration with CLI overrides
- **Structured Logging**: Comprehensive request/response logging with configurable levels

## Quick Start

### Build the Project

```bash
cargo build --release
```

### Run the Proxy

Basic usage (listens on localhost:8080):
```bash
cargo run
```

With custom port and connection limit:
```bash
cargo run -- --port 9000 --max-connections 200
```

With configuration file:
```bash
cargo run -- --config proxy.toml
```

### Test the Proxy

Using curl:
```bash
curl -x http://localhost:8080 http://httpbin.org/get
```

Using Python test script:
```bash
python test_proxy.py
```

## Configuration

Create a `proxy.toml` file:

```toml
[server]
host = "127.0.0.1"
port = 8080
max_connections = 100
timeout_seconds = 30

[upstream]
default_target = "httpbin.org:80"
```

## Command Line Options

- `-p, --port <PORT>`: Port to listen on (default: 8080)
- `-h, --host <HOST>`: Host to bind to (default: 127.0.0.1)
- `-t, --target <TARGET>`: Default target server URL
- `-l, --log-level <LEVEL>`: Log level (default: info)
- `--config <FILE>`: Configuration file path
- `--max-connections <NUM>`: Maximum concurrent connections (default: 100)

## Architecture

```
           +--------+       +------------+       +-----------+
           | Client | ----> | Tachyon    | ----> | Target    |
           | (curl) | <---- | Proxy      | <---- | Server    |
           +--------+       +------------+       +-----------+
```

1. **Client** sends HTTP request to the proxy
2. **Proxy** parses the request and forwards it to the target server
3. **Target Server** responds
4. **Proxy** relays the response back to the client

## Project Structure

```
src/
├── main.rs           # CLI setup and startup
├── proxy.rs          # Core proxy logic
├── http.rs           # HTTP parsing
└── config.rs         # Configuration management
```

## Performance Features

- **Async I/O**: Non-blocking network operations with Tokio
- **Connection Limiting**: Semaphore-based connection throttling
- **Timeout Management**: Prevents hanging connections
- **Memory Efficiency**: Bounded buffers and efficient parsing

## Learning Value

This project demonstrates:

- **Low-level Networking**: TCP socket management and HTTP protocol parsing
- **Concurrency**: Handling multiple client connections simultaneously
- **Systems Programming**: Memory management, error handling, and performance optimization
- **Rust Async**: Working with Tokio runtime and async/await patterns

## Development

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## License

MIT License 