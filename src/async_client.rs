use std::{error::Error, ops::Range, sync::Arc};

use tokio::{
    io::{
        AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, Error as IoError,
        Result as IoResult,
    },
    net::TcpStream,
    task::JoinHandle,
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
    addr: Arc<String>,
}
impl ServerConfig {
    pub fn new(addr: String) -> Self {
        Self {
            addr: Arc::new(addr),
        }
    }
}

pub struct AsyncClient {
    server_config: ServerConfig,
    data_config: DataConfig,
}
impl AsyncClient {
    const CHUNK_SIZE: u32 = 64 * KB;

    pub fn new(server_config: ServerConfig, data_config: DataConfig) -> Self {
        Self {
            server_config,
            data_config,
        }
    }

    pub async fn run(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut data: Vec<u8> = Vec::new(); // aggregated data

        let mut tasks: Vec<JoinHandle<Result<Vec<u8>, IoError>>> = Vec::new();

        let mut start = 0;
        let len = self.data_config.len;
        while start < len {
            let server_addr = Arc::clone(&self.server_config.addr);
            let end = len.min(start + Self::CHUNK_SIZE);

            // spawn a batch of tokio tasks
            tasks.push(tokio::spawn(async move {
                let mut data_chunk = Vec::new();

                // (re)connect because server closes connection
                let mut stream = TcpStream::connect(&*server_addr).await?;

                // send request
                let request = Self::get_request(start..end);
                stream.write_all(request.as_bytes()).await?;

                // receive response
                let response_body = Self::read_response(&mut stream).await?;
                data_chunk.extend_from_slice(&response_body);

                Ok(data_chunk)
            }));

            // move to the next chunk
            start += Self::CHUNK_SIZE;
        }

        // collect the results
        for task in tasks {
            let data_chunk = task.await??;

            data.extend_from_slice(&data_chunk);
        }

        // assert
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

    async fn read_response(stream: &mut TcpStream) -> IoResult<Vec<u8>> {
        let mut reader = BufReader::new(stream);

        // skip headers
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).await?;
            if line == "\r\n" {
                break;
            }
        }

        let mut body = Vec::new();
        reader.read_to_end(&mut body).await?;

        Ok(body)
    }
}
