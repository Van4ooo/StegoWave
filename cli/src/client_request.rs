use std::fs;
use stego_wave::error::StegoWaveClientError;
use stego_wave::object::StegoWaveClient;
use crate::cli::{ClearCommand, Cli, Commands, ExtractCommand, HideCommand, StegoWaveServer};

const GRPC_URL: &str = "http://[::1]:50051";
const REST_URL: &str = "http://127.0.0.1:8080";

pub async fn client_request(cli: Cli){
    match cli.commands {
        Commands::Hide(hide) => {
            match hide_command(hide).await{
                Ok(_) => println!("Сюда"),
                Err(err) => eprintln!("Error: {err}"),
            };
        },
        Commands::Extract(extract) => {
            let res = extract_command(extract).await;
            if res.is_ok(){

            }
        },
        Commands::Clear(clear) => {
            let res = clear_command(clear).await;
            if res.is_ok(){

            }
        },
    }
}

async fn hide_command(hide: HideCommand) -> Result<Vec<u8>, String> {
    let mut client = get_client(&hide.command.server).await
        .map_err(|_err| "Failed to get client".to_string())?;
    let file_byte = fs::read(&hide.command.file_name)
        .map_err(|_err| format!("Failed to read {:?}", hide.command.file_name))?;

    let result = client.hide_message(
        file_byte,
        hide.command.file_name.to_str().unwrap_or_default().to_string(),
        hide.message,
        hide.password,
        hide.command.format.into(),
        hide.command.lsb_deep
    ).await.map_err(|err| format!("{err}"))?;

    Ok(result)
}

async fn extract_command(_extract: ExtractCommand) -> Result<(), ()> {
    unimplemented!()
}

async fn clear_command(_clear: ClearCommand) -> Result<(), ()>{
    unimplemented!()
}

#[inline]
async fn get_client(server: &StegoWaveServer) -> Result<Box<dyn StegoWaveClient>, StegoWaveClientError>{
    match _get_client(server).await {
        Ok(client) => Ok(client),
        Err(StegoWaveClientError::ConnectionFailed) => {
            eprintln!("[!] Failed to connect to the server\n Exiting...");
            Err(StegoWaveClientError::ConnectionFailed)
        },
        Err(StegoWaveClientError::UlrInvalid) => {
            eprintln!("[!] The server address is invalid\n Exiting...");
            Err(StegoWaveClientError::UlrInvalid)
        },
        Err(err) => {
            eprintln!("[!] Failed to get client\n Exiting...");
            Err(err)
        }
    }
}

async fn _get_client(server: &StegoWaveServer) -> Result<Box<dyn StegoWaveClient>, StegoWaveClientError>{
    match server {
        StegoWaveServer::Auto => {
            if let Ok(client) = grpc_client::StegoWaveGrpcClient::new(GRPC_URL).await{
                Ok(Box::new(client))
            } else{
                eprintln!("[!] Failed to connect to gRPC server at {}", GRPC_URL);
                println!("[+] Trying connect to REST server at {}", REST_URL);
                let client = rest_client::StegoWaveRestClient::new(REST_URL).await?;
                Ok(Box::new(client))
            }
        },
        StegoWaveServer::GRPC => {
            let client = grpc_client::StegoWaveGrpcClient::new(GRPC_URL).await?;
            Ok(Box::new(client))
        },
        StegoWaveServer::REST => {
            let client = rest_client::StegoWaveRestClient::new(REST_URL).await?;
            Ok(Box::new(client))
        },
    }
}