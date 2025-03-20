mod services;

use stego_wave_grpc::stego_wave_service_server::StegoWaveServiceServer;
use tonic::transport::Server;

pub mod stego_wave_grpc {
    tonic::include_proto!("stego_wave");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let stego_wave_service = services::StegoWaveServiceImpl::default();

    Server::builder()
        .add_service(StegoWaveServiceServer::new(stego_wave_service))
        .serve(addr)
        .await?;

    Ok(())
}
