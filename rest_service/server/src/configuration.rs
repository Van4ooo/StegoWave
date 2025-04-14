use serde::Deserialize;
use std::mem;
use stego_wave::configuration::StegoWaveLib;
use stego_wave::error::StegoWaveClientError;
use url::Url;

#[derive(Deserialize)]
pub struct RestConfig {
    pub host: String,
    pub port: u32,
}

#[derive(Deserialize)]
pub struct Settings {
    pub rest: RestConfig,
    pub stego_wave_lib: StegoWaveLib,
}

impl RestConfig {
    pub fn address(self: &RestConfig) -> Result<Url, StegoWaveClientError> {
        Url::parse(&format!("http://{}:{}", self.host, self.port))
            .map_err(|err| StegoWaveClientError::UlrInvalid(err.to_string()))
    }
}

impl Settings {
    pub fn new(config_file: &str) -> Result<Self, config::ConfigError> {
        let conf = config::Config::builder()
            .add_source(config::File::with_name(config_file).required(true))
            .build()?;

        conf.try_deserialize()
    }

    pub fn get_stego_wave_lib_settings(&mut self) -> StegoWaveLib {
        mem::take(&mut self.stego_wave_lib)
    }
}
