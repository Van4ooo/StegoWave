use grpc_server::configuration;
use grpc_server::startup::run_server;

const CONFIG_FILE: &str = "sw_config.toml";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = configuration::Settings::new(CONFIG_FILE)?;
    let addr = settings.address().parse()?;

    run_server(addr, settings.stego_wave_lib).await?;
    Ok(())
}
