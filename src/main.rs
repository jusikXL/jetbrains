// mod async_client;
mod sync_client;

use std::{
    io::{self},
    process,
};

use sync_client::{DataConfig, ServerConfig, SyncClient};

const SERVER_ADDR: &str = "127.0.0.1:8080";

// #[tokio::main]
/* async */
fn main() {
    let data_config = read_config();

    let client = SyncClient::new(ServerConfig::new(SERVER_ADDR.to_string()), data_config);

    if let Err(e) = client.run()
    /* .await */
    {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}

fn read_config() -> DataConfig {
    println!("Please input data length.");
    let len: u32 = loop {
        let mut buf = String::new();

        if io::stdin().read_line(&mut buf).is_err() {
            println!("Try inputting length again.");
            continue;
        }

        match buf.trim().parse::<u32>() {
            Ok(num) => break num,
            Err(_) => {
                println!("Try inputting length again.");
                continue;
            }
        }
    };

    println!("Please input data hash.");
    let hash = loop {
        let mut buf = String::new();

        if io::stdin().read_line(&mut buf).is_err() {
            println!("Try inputting hash again.");
            continue;
        }

        let buf = buf.trim().to_string();

        if buf.len() != 64 {
            println!("Try inputting hash again.");
            continue;
        } else {
            break buf;
        }
    };

    println!("Fetching data of length {} with hash {}", len, hash);

    DataConfig::new(len, hash)
}
