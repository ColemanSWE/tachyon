# Setup Guide for Tachyon Proxy

## Prerequisites

### Installing Rust

1. **Windows**: Visit [https://rustup.rs/](https://rustup.rs/) and download the installer
   ```powershell
   # Run the installer and follow the prompts
   # Then restart your terminal
   ```

2. **macOS/Linux**: Use rustup
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

3. **Verify Installation**:
   ```bash
   rustc --version
   cargo --version
   ```

## Building and Running

### 1. Build the Project
```bash
cargo build --release
```

### 2. Run the Proxy
```bash
# Basic usage (listens on localhost:8080)
cargo run

# With custom port
cargo run -- --port 9000

# With specific target server
cargo run -- --port 8080 --target http://example.com:80
```

### 3. Test the Proxy

**Using curl:**
```bash
curl -x http://localhost:8080 http://httpbin.org/get
```

**Using Python test script:**
```bash
python test_proxy.py
```

**Using wget:**
```bash
wget --proxy=on --proxy-type=http --proxy-host=localhost --proxy-port=8080 http://httpbin.org/get
```

## Development Commands

```bash
# Check for compilation errors
cargo check

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy

# Build optimized release
cargo build --release
```

## Troubleshooting

### Common Issues

1. **"cargo: command not found"**
   - Make sure Rust is installed and PATH is updated
   - Restart your terminal after installation

2. **"Address already in use"**
   - Change the port: `cargo run -- --port 9000`
   - Or kill the process using the port

3. **Proxy connection refused**
   - Ensure the proxy is running before testing
   - Check firewall settings

4. **Build errors**
   - Update Rust: `rustup update`
   - Clean and rebuild: `cargo clean && cargo build`

## Performance Tips

- Use `--release` flag for production builds
- Monitor memory usage with large connection loads
- Consider adjusting `max_connections` in the code for your use case 