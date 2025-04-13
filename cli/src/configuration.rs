use grpc_server::configuration::GrpcConfig;
use rest_server::configuration::RestConfig;
use serde::Deserialize;
use stego_wave::configuration::StegoWaveLib;

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
}
