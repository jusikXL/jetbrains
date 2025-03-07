use std::{
    error::Error,
    io::{self, BufRead, BufReader, Read, Write},
    net::TcpStream,
    ops::Range,
    process,
};

const SERVER_ADDR: &str = "127.0.0.1:8080";
const KB: u32 = 1024;
const CHUNK_SIZE: u32 = 64 * KB;

struct DataConfig {
    len: u32, // could have utilized 'Content-Range' header, but it is not provided by the server
    hash: String,
}

struct ServerConfig {
    addr: String,
}

struct Client {
    server_config: ServerConfig,
    data_config: DataConfig,
}

impl Client {
    pub fn new(server_config: ServerConfig, data_config: DataConfig) -> Self {
        Self {
            server_config,
            data_config,
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let mut data: Vec<u8> = Vec::new(); // aggregated data

        let mut start = 0;
        let len = self.data_config.len;
        while start < len {
            let end = len.min(start + CHUNK_SIZE);

            // (re)connect as server closes connection
            let mut stream = TcpStream::connect(&self.server_config.addr)?;

            // send request
            let request = Self::get_request(start..end);
            stream.write_all(request.as_bytes())?;

            // receive response
            let response_body = Self::read_response(&mut stream)?;
            data.extend_from_slice(&response_body);

            // move to the next chunk
            start += CHUNK_SIZE;
        }

        assert_eq!(data.len() as u32, len, "length mismatch");
        assert_eq!(sha256::digest(data), self.data_config.hash, "hash mismatch");

        println!("Success");
        Ok(())
    }

    fn get_request(range: Range<u32>) -> String {
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
}

fn main() {
    let data_config = read_config();

    println!(
        "Fetching data of length {} with hash {}",
        data_config.len, data_config.hash
    );

    let client = Client::new(
        ServerConfig {
            addr: SERVER_ADDR.to_string(),
        },
        data_config,
    );

    if let Err(e) = client.run() {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}

fn read_config() -> DataConfig {
    println!("Please input data length.");
    let len: u32 = loop {
        let mut buf = String::new();

        if io::stdin().read_line(&mut buf).is_err() {
            println!("Try inputing length again.");
            continue;
        }

        match buf.trim().parse::<u32>() {
            Ok(num) => break num,
            Err(_) => {
                println!("Try inputing length again.");
                continue;
            }
        }
    };

    println!("Please input data hash.");
    let hash = loop {
        let mut buf = String::new();

        if io::stdin().read_line(&mut buf).is_err() {
            println!("Try inputing hash again.");
            continue;
        }

        let buf = buf.trim().to_string();

        if buf.len() != 64 {
            println!("Try inputing hash again.");
            continue;
        } else {
            break buf;
        }
    };

    DataConfig { len, hash }
}
