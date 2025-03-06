pub struct DataConfig {
    pub len: i32, // could have utilized 'Content-Range' header, but it is not provided by the server
    pub hash: String,
}

pub struct Config {
    pub data: DataConfig,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Self, &'static str> {
        args.next();

        let len = args
            .next()
            .ok_or("Didn't get data length")?
            .parse::<i32>()
            .map_err(|_| "Could not parse data length")?;

        let hash = args.next().ok_or("Didn't get data hash")?;

        Ok(Self {
            data: DataConfig { len, hash },
        })
    }
}
