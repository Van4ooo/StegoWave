use crate::cli::{ClearCommand, Cli, Commands, ExtractCommand, HideCommand, StegoWaveServer};
use crate::formating::{print_error, print_format, print_info, print_success};
use colored::*;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use stego_wave::error::StegoWaveClientError;
use stego_wave::object::StegoWaveClient;

const GRPC_URL: &str = "http://[::1]:50051";
const REST_URL: &str = "http://127.0.0.1:8080";
const FILE_PREFIX: &str = "sw_";

pub async fn client_request(cli: Cli) {
    match cli.commands {
        Commands::Hide(hide) => {
            if let Err(err) = hide_command(hide).await {
                print_error(err);
            };
        }
        Commands::Extract(extract) => {
            if let Err(err) = extract_command(extract).await {
                print_error(err);
            }
        }
        Commands::Clear(clear) => {
            if let Err(err) = clear_command(clear).await {
                print_error(err);
            }
        }
    }
}

async fn hide_command(hide: HideCommand) -> Result<(), String> {
    let mut client = get_client(&hide.command.server).await?;
    let file_byte = get_file_by_name(&hide.command.file_name)?;

    let result: Vec<u8> = client
        .hide_message(
            file_byte,
            hide.command
                .file_name
                .to_str()
                .unwrap_or_default()
                .to_string(),
            hide.message,
            hide.password,
            hide.command.format.into(),
            hide.command.lsb_deep,
        )
        .await
        .map_err(map_client_error)?;

    let output_file = add_prefix(&hide.command.file_name, FILE_PREFIX);
    write_response_to_file(&output_file, &result)?;

    print_format("input file", "output file");
    print_success(hide.command.file_name, output_file);

    Ok(())
}

async fn extract_command(extract: ExtractCommand) -> Result<(), String> {
    let mut client = get_client(&extract.command.server).await?;
    let file_byte = get_file_by_name(&extract.command.file_name)?;

    let result: String = client
        .extract_message(
            file_byte,
            extract
                .command
                .file_name
                .to_str()
                .unwrap_or_default()
                .to_string(),
            extract.password,
            extract.command.format.into(),
            extract.command.lsb_deep,
        )
        .await
        .map_err(map_client_error)?;

    print_format("input file", "secret message");
    print_success(extract.command.file_name, result);

    Ok(())
}

async fn clear_command(clear: ClearCommand) -> Result<(), String> {
    let mut client = get_client(&clear.command.server).await?;
    let file_byte = get_file_by_name(&clear.command.file_name)?;

    let result: Vec<u8> = client
        .clear_message(
            file_byte,
            clear
                .command
                .file_name
                .to_str()
                .unwrap_or_default()
                .to_string(),
            clear.password,
            clear.command.format.into(),
            clear.command.lsb_deep,
        )
        .await
        .map_err(map_client_error)?;

    let output_file = add_prefix(&clear.command.file_name, "_");
    write_response_to_file(&output_file, &result)?;

    print_format("input file", "cleared file");
    print_success(clear.command.file_name, output_file);

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

#[inline]
async fn get_client(server: &StegoWaveServer) -> Result<Box<dyn StegoWaveClient>, String> {
    match _get_client(server).await {
        Ok(client) => Ok(client),
        Err(StegoWaveClientError::ConnectionFailed) => {
            Err("Failed to connect to the server".to_string())
        }
        Err(StegoWaveClientError::UlrInvalid) => Err("The server address is invalid".to_string()),
        Err(err) => Err(format!("Failed to get client\nError: {err}")),
    }
}

async fn _get_client(
    server: &StegoWaveServer,
) -> Result<Box<dyn StegoWaveClient>, StegoWaveClientError> {
    match server {
        StegoWaveServer::Auto => {
            if let Ok(client) = grpc_client::StegoWaveGrpcClient::new(GRPC_URL).await {
                Ok(Box::new(client))
            } else {
                print_error(format!(
                    "Failed to connect to gRPC server at {}",
                    GRPC_URL.blue()
                ));
                print_info(format!(
                    "Trying connect to REST server at {}",
                    REST_URL.blue()
                ));
                let client = rest_client::StegoWaveRestClient::new(REST_URL).await?;
                Ok(Box::new(client))
            }
        }
        StegoWaveServer::GRPC => {
            let client = grpc_client::StegoWaveGrpcClient::new(GRPC_URL).await?;
            Ok(Box::new(client))
        }
        StegoWaveServer::REST => {
            let client = rest_client::StegoWaveRestClient::new(REST_URL).await?;
            Ok(Box::new(client))
        }
    }
}

#[inline]
fn get_file_by_name(file_name: &PathBuf) -> Result<Vec<u8>, String> {
    fs::read(file_name).map_err(|_err| format!("Failed to read {:?}", file_name))
}

#[inline]
fn map_client_error(err: StegoWaveClientError) -> String {
    format!("{err}")
}

fn write_response_to_file(input_file: &Path, result: &[u8]) -> Result<(), String> {
    let mut file =
        File::create(input_file).map_err(|err| format!("Failed to create output file :: {err}"))?;
    file.write_all(result)
        .map_err(|_err| format!("Failed to write byte to {:?}", input_file))?;

    Ok(())
}
