# Rust CLI Client  

A command-line client written in Rust that fetches data from a glitchy HTTP server (`main.py`) and verifies data integrity using SHA-256 (`sha2` crate).  

## Usage  

Run the client:  
```sh
cargo run
```  
Follow the on-screen instructions.  

## Implementation  

The client retrieves data in 64 KB chunks to mitigate corruption caused by the glitchy server.  

- **Synchronous Client (`sync_client.rs`)**  
  Fetches data chunk-by-chunk sequentially.  

- **Asynchronous Client (`async_client.rs`)**
  > (received your email just the moment I finished implementing it)
  Fetches and processes multiple chunks concurrently for better performance.  
  - To test, uncomment the `tokio` dependency in `Cargo.toml`.  
  - Modify `main.rs` as instructed by the compiler.  
