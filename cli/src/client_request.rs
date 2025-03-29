use crate::cli::{ClearCommand, Cli, Commands, ExtractCommand, HideCommand, StegoWaveServer};
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
            match hide_command(hide).await {
                Ok(_) => {}
                Err(err) => eprintln!("{err}"),
            };
        }
        Commands::Extract(extract) => {
            let res = extract_command(extract).await;
            if res.is_ok() {}
        }
        Commands::Clear(clear) => {
            let res = clear_command(clear).await;
            if res.is_ok() {}
        }
    }
}

async fn hide_command(hide: HideCommand) -> Result<(), String> {
    let mut client = get_client(&hide.command.server)
        .await
        .map_err(|_err| "Failed to get client".to_string())?;

    let file_byte = fs::read(&hide.command.file_name)
        .map_err(|_err| format!("Failed to read {:?}", hide.command.file_name))?;

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
        .map_err(|err| format!("{err}"))?;

    let output_file = add_prefix(&hide.command.file_name, FILE_PREFIX);

    let mut file = File::create(&output_file)
        .map_err(|err| format!("Failed to create output file :: {err}"))?;
    file.write_all(&result)
        .map_err(|_err| format!("Failed to write byte to {:?}", output_file))?;

    println!(
        "[SUCCESS] {:?} -> {:?}",
        hide.command.file_name, output_file
    );
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

async fn extract_command(_extract: ExtractCommand) -> Result<(), ()> {
    unimplemented!()
}

async fn clear_command(_clear: ClearCommand) -> Result<(), ()> {
    unimplemented!()
}

#[inline]
async fn get_client(
    server: &StegoWaveServer,
) -> Result<Box<dyn StegoWaveClient>, StegoWaveClientError> {
    match _get_client(server).await {
        Ok(client) => Ok(client),
        Err(StegoWaveClientError::ConnectionFailed) => {
            eprintln!("[!] Failed to connect to the server\nExiting...");
            Err(StegoWaveClientError::ConnectionFailed)
        }
        Err(StegoWaveClientError::UlrInvalid) => {
            eprintln!("[!] The server address is invalid\nExiting...");
            Err(StegoWaveClientError::UlrInvalid)
        }
        Err(err) => {
            eprintln!("[!] Failed to get client\nExiting...");
            Err(err)
        }
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
                eprintln!("[!] Failed to connect to gRPC server at {}", GRPC_URL);
                println!("[+] Trying connect to REST server at {}", REST_URL);
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
