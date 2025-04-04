use serde::Deserialize;
use stego_wave::configuration::StegoWaveLib;

#[derive(Deserialize)]
pub struct GrpcConfig {
    pub host: String,
    pub port: u32,
}

#[derive(Deserialize)]
pub struct Settings {
    pub grpc: GrpcConfig,
    pub stego_wave_lib: StegoWaveLib,
}

impl Settings {
    pub fn new(config_file: &str) -> Result<Self, config::ConfigError> {
        let conf = config::Config::builder()
            .add_source(config::File::with_name(config_file).required(true))
            .add_source(config::Environment::with_prefix("SW__GRPC").separator("__"))
            .build()?;

        conf.try_deserialize()
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.grpc.host, self.grpc.port)
    }
}
