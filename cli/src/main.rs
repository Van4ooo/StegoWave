use colored::Colorize;

mod cli;
mod client_request;
mod configuration;
mod formating;
mod startup;

const CONFIG_FILE: &str = "sw_config.toml";

#[tokio::main]
async fn main() {
    if let Err(report) = startup::run().await {
        eprintln!("{}: {:?}", "Failed".red(), report);
    }
}
