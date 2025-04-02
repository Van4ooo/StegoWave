use crate::configuration::Settings;
use crate::{CONFIG_FILE, cli, client_request};
use clap::Parser;
use color_eyre::Section;

pub async fn run() -> color_eyre::Result<()> {
    color_eyre::config::HookBuilder::default()
        .display_env_section(false)
        .install()?;

    let cli_args = cli::Cli::parse();
    let settings = Settings::new(CONFIG_FILE)
        .suggestion(format!("Please create configuration file {CONFIG_FILE}"))?;

    client_request::client_request(cli_args, settings).await
}
