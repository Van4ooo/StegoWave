use clap::Parser;

mod cli;
mod client_request;

#[tokio::main]
async fn main() {
    let cli_args: cli::Cli = cli::Cli::parse();
    client_request::client_request(cli_args).await;
}
