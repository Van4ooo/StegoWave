use crate::cli::{ClearCommand, Cli, Commands, ExtractCommand, HideCommand, StegoWaveServer};
use crate::configuration::Settings;
use crate::formating::print_success_helper;
use crate::print_success;
use color_eyre::eyre::Context;
use color_eyre::{Report, Result, Section};
use colored::Colorize;
use std::io;
use std::io::{Write, stderr};
use std::path::PathBuf;
use stego_wave::error::StegoWaveClientError;
use stego_wave::object::StegoWaveClient;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn client_request(cli: Cli, settings: Settings) -> Result<()> {
    match cli.commands {
        Commands::Hide(hide) => hide_command(hide, settings)
            .await
            .wrap_err("Error executing <hide> command"),
        Commands::Extract(extract) => extract_command(extract, settings)
            .await
            .wrap_err("Error executing <extract> command"),
        Commands::Clear(clear) => clear_command(clear, settings)
            .await
            .wrap_err("Error executing <clear> command"),
    }
}

async fn hide_command(hide: HideCommand, settings: Settings) -> Result<()> {
    let mut client = get_client(&hide.command.server, &settings)
        .await
        .map_err(stego_client_wrap_error)?;

    let file_byte = get_input_file(&hide.command.input_file).await?;
    let password = read_user_password()?;

    let result: Vec<u8> = client
        .hide_message(
            file_byte,
            hide.message,
            password,
            hide.command.format.into(),
            hide.command.lsb_deep,
        )
        .await?;

    write_response_to_file(&hide.output_file, &result).await?;

    Ok(())
}

async fn extract_command(extract: ExtractCommand, settings: Settings) -> Result<()> {
    let mut client = get_client(&extract.command.server, &settings)
        .await
        .map_err(stego_client_wrap_error)?;

    let file_byte = get_input_file(&extract.command.input_file).await?;
    let password = read_user_password()?;

    let result: String = client
        .extract_message(
            file_byte,
            password,
            extract.command.format.into(),
            extract.command.lsb_deep,
        )
        .await?;

    print_success!(message: result);
    Ok(())
}

async fn clear_command(clear: ClearCommand, settings: Settings) -> Result<()> {
    let mut client = get_client(&clear.command.server, &settings)
        .await
        .map_err(stego_client_wrap_error)?;

    let file_byte = get_input_file(&clear.command.input_file).await?;
    let password = read_user_password()?;

    let result: Vec<u8> = client
        .clear_message(
            file_byte,
            password,
            clear.command.format.into(),
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
    let grpc_address = settings.grpc_address();
    let rest_address = settings.rest_address();

    match server {
        StegoWaveServer::Auto => {
            if let Ok(client) = grpc_client::StegoWaveGrpcClient::new(grpc_address.as_str()).await {
                Ok(Box::new(client))
            } else {
                let client = rest_client::StegoWaveRestClient::new(rest_address.as_str()).await?;
                Ok(Box::new(client))
            }
        }
        StegoWaveServer::GRPC => {
            let client = grpc_client::StegoWaveGrpcClient::new(grpc_address.as_str()).await?;
            Ok(Box::new(client))
        }
        StegoWaveServer::REST => {
            let client = rest_client::StegoWaveRestClient::new(rest_address.as_str()).await?;
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
