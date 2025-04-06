use rest_server::{configuration, startup::run_server, tracing_config};
use std::net::TcpListener;

const CONFIG_FILE: &str = "sw_config.toml";

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_config::conf_logger();

    let mut settings = configuration::Settings::new(CONFIG_FILE)?;
    let stego_wave_setting = settings.get_stego_wave_lib_settings();
    let listener = TcpListener::bind(settings.address())?;

    run_server(listener, stego_wave_setting)?.await?;

    Ok(())
}
