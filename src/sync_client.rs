use std::{
    error::Error,
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
    ops::Range,
};

const KB: u32 = 1024;

pub struct DataConfig {
    len: u32, // could have utilized 'Content-Range' header, but it is not provided by the server
    hash: String,
}
impl DataConfig {
    pub fn new(len: u32, hash: String) -> Self {
        Self { len, hash }
    }
}

pub struct ServerConfig {
    addr: String,
}
impl ServerConfig {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}

pub struct Client {
    server_config: ServerConfig,
    data_config: DataConfig,
}

impl Client {
    const CHUNK_SIZE: u32 = 64 * KB;

    pub fn new(server_config: ServerConfig, data_config: DataConfig) -> Self {
        Self {
            server_config,
            data_config,
        }
    }

    pub fn run(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut data: Vec<u8> = Vec::new(); // aggregated data

        let mut start = 0;
        let len = self.data_config.len;
        while start < len {
            let end = len.min(start + Self::CHUNK_SIZE);

            // (re)connect as server closes connection
            let mut stream = TcpStream::connect(&self.server_config.addr)?;

            // send request
            let request = Self::get_request(start..end);
            stream.write_all(request.as_bytes())?;

            // receive response
            let response_body = Self::read_response(&mut stream)?;
            data.extend_from_slice(&response_body);

            // move to the next chunk
            start += Self::CHUNK_SIZE;
        }

        assert_eq!(data.len() as u32, len, "length mismatch");
        assert_eq!(
            sha256::digest(&data),
            self.data_config.hash,
            "hash mismatch"
        );

        println!("Success");
        Ok(data)
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
