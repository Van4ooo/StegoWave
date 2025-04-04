use serde::Deserialize;

const CONFIG_FILE: &str = "sw_config.toml";

#[derive(Deserialize, Debug, PartialEq, Clone, Default)]
pub struct StegoWaveLib {
    pub header: String,
    pub default_lsb_deep: u8,
    pub max_occupancy: usize,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Settings {
    pub stego_wave_lib: StegoWaveLib,
}

impl Settings {
    pub fn new(config_file: &str) -> Result<Self, config::ConfigError> {
        let conf = config::Config::builder()
            .add_source(config::File::with_name(config_file).required(true))
            .add_source(config::Environment::with_prefix("STEGO_WAVE_LIB").separator("__"))
            .build()?;

        conf.try_deserialize()
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new(CONFIG_FILE)
            .expect("Failed to build configuration from file and environment for stego_wave")
    }
}
