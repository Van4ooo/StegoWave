use crate::cli::{ClearCommand, Cli, Commands, ExtractCommand, HideCommand, StegoWaveServer};
use crate::configuration::Settings;
use crate::formating::{print_format, print_input_path};
use crate::print_success;
use color_eyre::eyre::Context;
use color_eyre::{Report, Result, Section};
use colored::Colorize;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};
use stego_wave::error::StegoWaveClientError;
use stego_wave::object::StegoWaveClient;

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

    let file_byte =
        fs::read(&hide.command.file_name).suggestion("Try using a file that exists next time")?;
    let password = read_user_password()?;

    let result: Vec<u8> = client
        .hide_message(
            file_byte,
            hide.command
                .file_name
                .to_str()
                .unwrap_or_default()
                .to_string(),
            hide.message,
            password,
            hide.command.format.into(),
            hide.command.lsb_deep,
        )
        .await?;

    let output_file = add_prefix(&hide.command.file_name, &settings.sw.hide_file_prefix);
    write_response_to_file(&output_file, &result)?;
    print_success!(file: output_file, hide.command.file_name);

    Ok(())
}

async fn extract_command(extract: ExtractCommand, settings: Settings) -> Result<()> {
    let mut client = get_client(&extract.command.server, &settings)
        .await
        .map_err(stego_client_wrap_error)?;

    let file_byte = fs::read(&extract.command.file_name)
        .suggestion("Try using a file that exists next time")?;

    let password = read_user_password()?;

    let result: String = client
        .extract_message(
            file_byte,
            extract
                .command
                .file_name
                .to_str()
                .unwrap_or_default()
                .to_string(),
            password,
            extract.command.format.into(),
            extract.command.lsb_deep,
        )
        .await?;

    print_success!(message: result, extract.command.file_name);
    Ok(())
}

async fn clear_command(clear: ClearCommand, settings: Settings) -> Result<()> {
    let mut client = get_client(&clear.command.server, &settings)
        .await
        .map_err(stego_client_wrap_error)?;

    let file_byte =
        fs::read(&clear.command.file_name).suggestion("Try using a file that exists next time")?;
    let password = read_user_password()?;

    let result: Vec<u8> = client
        .clear_message(
            file_byte,
            clear
                .command
                .file_name
                .to_str()
                .unwrap_or_default()
                .to_string(),
            password,
            clear.command.format.into(),
            clear.command.lsb_deep,
        )
        .await?;

    let output_file = add_prefix(&clear.command.file_name, &settings.sw.clear_file_prefix);
    write_response_to_file(&output_file, &result)?;
    print_success!(file: output_file, clear.command.file_name);

    Ok(())
}

fn add_prefix(file_path: &Path, prefix: &str) -> PathBuf {
    let parent = file_path.parent().unwrap_or_else(|| Path::new(""));
    let file_stem = file_path.file_stem().unwrap_or_default().to_string_lossy();
    let extension = file_path
        .extension()
        .map(|ext| format!(".{}", ext.to_string_lossy()))
        .unwrap_or_default();

    parent.join(format!("{}{}{}", prefix, file_stem, extension))
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

fn write_response_to_file(input_file: &Path, result: &[u8]) -> Result<()> {
    let mut file = File::create(input_file)?;
    file.write_all(result)?;

    Ok(())
}

fn read_user_password() -> Result<String, io::Error> {
    eprint!("[?] Enter password: ");
    let password = rpassword::read_password()?;

    Ok(password)
}
