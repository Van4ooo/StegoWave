use crate::cli::{ClearCommand, Cli, Commands, ExtractCommand, HideCommand, StegoWaveServer};
use crate::configuration::Settings;
use crate::formating::print_success_helper;
use crate::print_success;
use color_eyre::eyre::eyre;
use color_eyre::{Report, Result, Section};
use colored::Colorize;
use std::io;
use std::io::{Write, stderr};
use std::net::SocketAddr;
use std::net::TcpListener;
use std::path::PathBuf;
use std::time::Duration;
use stego_wave::error::StegoWaveClientError;
use stego_wave::object::StegoWaveClient;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::sleep;

const QUERY_ATTEMPTS: u8 = 2;

pub async fn client_request(cli: Cli, settings: Settings) -> Result<()> {
    let password = read_user_password()?;
    let file_bytes = get_input_file(cli.input_file()).await?;

    match query_attempt(&cli, &settings, &password, &file_bytes).await {
        Ok(()) => Ok(()),
        Err(err) if err.to_string() == "Connection failed" => {
            run_server(&cli, &settings).await?;
            query_attempt_with_sleep(&cli, &settings, &password, &file_bytes, QUERY_ATTEMPTS).await
        }
        Err(err) => Err(err),
    }
}

pub async fn query_attempt_with_sleep(
    cli: &Cli,
    settings: &Settings,
    password: &str,
    file_bytes: &[u8],
    attempt: u8,
) -> Result<()> {
    for _ in 0..attempt {
        match query_attempt(cli, settings, password, file_bytes).await {
            Ok(()) => return Ok(()),
            Err(err) if err.to_string() == "Connection failed" => {
                sleep(Duration::from_secs(2)).await;
            }
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

pub async fn query_attempt(
    cli: &Cli,
    settings: &Settings,
    password: &str,
    file_bytes: &[u8],
) -> Result<()> {
    match cli.command() {
        Commands::Hide(hide) => hide_command(hide, settings, password, file_bytes).await,
        Commands::Extract(extract) => {
            extract_command(extract, settings, password, file_bytes).await
        }
        Commands::Clear(clear) => clear_command(clear, settings, password, file_bytes).await,
    }
}

pub async fn run_server(cli: &Cli, settings: &Settings) -> Result<()> {
    if !cli.start_server() {
        return Err(eyre!("Failed to connect to the servers.")
            .suggestion("Try using the '--start-server' flag and the program will start the server automatically"));
    }

    match cli.server() {
        StegoWaveServer::Auto => {
            return Err(
                eyre!("Automatic server selection is not supported.").suggestion(
                    "Specify a server explicitly using '--server grpc' or '--server rest'",
                ),
            );
        }
        StegoWaveServer::GRPC => {
            let addr: SocketAddr = settings.grpc_address()?.authority().parse()?;

            drop(tokio::spawn(grpc_server::startup::run_server(
                addr,
                settings.stego_wave_lib.clone(),
            )));
        }
        StegoWaveServer::REST => {
            let listener: TcpListener = TcpListener::bind(settings.rest_address()?.authority())?;

            let server =
                rest_server::startup::run_server(listener, settings.stego_wave_lib.clone())?;

            drop(tokio::spawn(server));
        }
    }

    Ok(())
}

async fn hide_command(
    hide: &HideCommand,
    settings: &Settings,
    password: &str,
    file_bytes: &[u8],
) -> Result<()> {
    let mut client = get_client(&hide.command.server, settings)
        .await
        .map_err(stego_client_wrap_error)?;

    let result: Vec<u8> = client
        .hide_message(
            file_bytes.to_vec(),
            hide.message.clone(),
            password.to_string(),
            hide.command.format.clone().into(),
            hide.command.lsb_deep,
        )
        .await?;

    write_response_to_file(&hide.output_file, &result).await?;

    Ok(())
}

async fn extract_command(
    extract: &ExtractCommand,
    settings: &Settings,
    password: &str,
    file_bytes: &[u8],
) -> Result<()> {
    let mut client = get_client(&extract.command.server, settings)
        .await
        .map_err(stego_client_wrap_error)?;

    let result: String = client
        .extract_message(
            file_bytes.to_vec(),
            password.to_string(),
            extract.command.format.clone().into(),
            extract.command.lsb_deep,
        )
        .await?;

    print_success!(message: result);
    Ok(())
}

async fn clear_command(
    clear: &ClearCommand,
    settings: &Settings,
    password: &str,
    file_bytes: &[u8],
) -> Result<()> {
    let mut client = get_client(&clear.command.server, settings)
        .await
        .map_err(stego_client_wrap_error)?;

    let result: Vec<u8> = client
        .clear_message(
            file_bytes.to_vec(),
            password.to_string(),
            clear.command.format.clone().into(),
            clear.command.lsb_deep,
        )
        .await?;

    write_response_to_file(&clear.output_file, &result).await?;
    Ok(())
}

fn stego_client_wrap_error(err: StegoWaveClientError) -> Report {
    if let Some(help_message) = err.help_message() {
        Report::msg(err.to_string()).suggestion(help_message)
    } else {
        Report::msg(err.to_string())
    }
}

async fn get_client(
    server: &StegoWaveServer,
    settings: &Settings,
) -> Result<Box<dyn StegoWaveClient>, StegoWaveClientError> {
    let grpc_address = settings.grpc_address()?;
    let rest_address = settings.rest_address()?;

    match server {
        StegoWaveServer::Auto => {
            if let Ok(client) =
                grpc_client::StegoWaveGrpcClient::new(grpc_address.to_string()).await
            {
                Ok(Box::new(client))
            } else {
                let client = rest_client::StegoWaveRestClient::new(rest_address).await?;
                Ok(Box::new(client))
            }
        }
        StegoWaveServer::GRPC => {
            let client = grpc_client::StegoWaveGrpcClient::new(grpc_address.to_string()).await?;
            Ok(Box::new(client))
        }
        StegoWaveServer::REST => {
            let client = rest_client::StegoWaveRestClient::new(rest_address).await?;
            Ok(Box::new(client))
        }
    }
}

fn read_user_password() -> Result<String, io::Error> {
    eprint!("[?] Enter password: ");
    stderr().flush()?;

    let password = rpassword::read_password()?;

    Ok(password)
}

async fn get_input_file(input_file: &Option<PathBuf>) -> Result<Vec<u8>> {
    if let Some(file_name) = input_file {
        tokio::fs::read(file_name).await.suggestion(
            "Please either specify the --input_file option or pipe the audio file as input.",
        )
    } else {
        let mut audio_data = Vec::new();
        let mut stdin = tokio::io::stdin();
        stdin.read_to_end(&mut audio_data).await.suggestion(
            "Please either specify the --input_file option or pipe the audio file as input.",
        )?;

        Ok(audio_data)
    }
}

async fn write_response_to_file(output_file: &Option<PathBuf>, result: &[u8]) -> Result<()> {
    if let Some(output_file) = output_file {
        let mut file = tokio::fs::File::create(output_file).await?;
        file.write_all(result).await?;

        print_success!(file: output_file.display());
    } else {
        let mut stdout = tokio::io::stdout();
        stdout.write_all(result).await?;
        stdout.flush().await?;
    }
    Ok(())
}
