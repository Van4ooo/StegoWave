use serde::Deserialize;

#[derive(Deserialize)]
pub struct GrpcConfig {
    pub host: String,
    pub port: u32,
}

#[derive(Deserialize)]
pub struct RestConfig {
    pub host: String,
    pub port: u32,
}

#[derive(Deserialize)]
pub struct Settings {
    pub rest: RestConfig,
    pub grpc: GrpcConfig,
}

impl Settings {
    pub fn new(config_file: &str) -> Result<Self, config::ConfigError> {
        let conf = config::Config::builder()
            .add_source(config::File::with_name(config_file).required(true))
            .add_source(config::Environment::with_prefix("SW").separator("__"))
            .build()?;

        conf.try_deserialize()
    }

    pub fn grpc_address(&self) -> String {
        format!("http://{}:{}", self.grpc.host, self.grpc.port)
    }

    pub fn rest_address(&self) -> String {
        format!("http://{}:{}", self.rest.host, self.rest.port)
    }
}
