use serde::Deserialize;
use std::mem;
use stego_wave::configuration::StegoWaveLib;

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

impl Settings {
    pub fn new(config_file: &str) -> Result<Self, config::ConfigError> {
        let conf = config::Config::builder()
            .add_source(config::File::with_name(config_file).required(true))
            .build()?;

        conf.try_deserialize()
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.rest.host, self.rest.port)
    }

    pub fn get_stego_wave_lib_settings(&mut self) -> StegoWaveLib {
        mem::take(&mut self.stego_wave_lib)
    }
}
