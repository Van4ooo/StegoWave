use grpc_server::configuration::GrpcConfig;
use rest_server::configuration::RestConfig;
use serde::Deserialize;
use stego_wave::configuration::StegoWaveLib;
use stego_wave::error::StegoWaveClientError;
use url::Url;

#[derive(Deserialize)]
pub struct Settings {
    pub rest: RestConfig,
    pub grpc: GrpcConfig,
    pub stego_wave_lib: StegoWaveLib,
}

impl Settings {
    pub fn new(config_file: &str) -> Result<Self, config::ConfigError> {
        let conf = config::Config::builder()
            .add_source(config::File::with_name(config_file).required(true))
            .add_source(config::Environment::with_prefix("SW").separator("__"))
            .build()?;

        conf.try_deserialize()
    }

    pub fn grpc_address(&self) -> Result<Url, StegoWaveClientError> {
        Url::parse(&format!("http://{}:{}", self.grpc.host, self.grpc.port))
            .map_err(|err| StegoWaveClientError::UlrInvalid(err.to_string()))
    }

    pub fn rest_address(&self) -> Result<Url, StegoWaveClientError> {
        Url::parse(&format!("http://{}:{}", self.rest.host, self.rest.port))
            .map_err(|err| StegoWaveClientError::UlrInvalid(err.to_string()))
    }
}
