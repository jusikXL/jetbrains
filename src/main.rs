mod config;

use std::{
    env,
    error::Error,
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
    ops::Range,
    process,
};

use config::Config;

const SERVER_ADDR: &str = "127.0.0.1:8080";
const KB: i32 = 1024;
const CHUNK_SIZE: i32 = 64 * KB;

// TODO: refactor, improve error handling, introduce concurrency
// ask whether it is allowed to use tokio for concurrency
// otherwise threads pool can be used

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|e| {
        eprintln!("Config error: {e}");
        process::exit(1);
    });

    println!(
        "Fetching data of len {} with hash {}",
        config.data.len, config.data.hash
    );

    if let Err(e) = run(&config) {
        println!("Application error: {e}");
        process::exit(1);
    }
}

fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    let mut start = 0;
    let len = config.data.len;

    let mut data: Vec<u8> = Vec::new();

    while start < len {
        let mut stream = TcpStream::connect(SERVER_ADDR)?; // reconnect

        let end = len.min(start + CHUNK_SIZE);

        let request = get_request(start..end);

        stream.write_all(request.as_bytes())?;

        let response_body = read_response(&mut stream)?;
        data.extend_from_slice(&response_body);

        start += CHUNK_SIZE;
    }

    println!("Total data received: {} bytes", data.len());

    let hash = sha256::digest(data);
    println!("{hash}");
    assert_eq!(hash, config.data.hash);

    Ok(())
}

fn get_request(range: Range<i32>) -> String {
    println!("{}-{}", range.start, range.end - 1);
    format!(
        "GET / HTTP/1.1\r\nRange: bytes={}-{}\r\n\r\n",
        range.start, range.end
    )
}

fn read_response(stream: &mut TcpStream) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut reader = BufReader::new(stream);

    // skip headers
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        if line == "\r\n" {
            break;
        }
    }

    let mut body = Vec::new();
    reader.read_to_end(&mut body)?;

    Ok(body)
}
