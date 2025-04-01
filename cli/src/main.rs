use crate::configuration::Settings;
use clap::Parser;

mod cli;
mod client_request;
mod configuration;
mod formating;

const CONFIG_FILE: &str = "sw_config.toml";

#[tokio::main]
async fn main() -> Result<(), config::ConfigError> {
    let cli_args: cli::Cli = cli::Cli::parse();
    let settings = Settings::new(CONFIG_FILE)?;

    client_request::client_request(cli_args, settings).await;

    Ok(())
}
