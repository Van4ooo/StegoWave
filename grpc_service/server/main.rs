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

    let svc = StegoWaveServiceServer::new(stego_wave_service);

    Server::builder()
        .max_frame_size(8 * 1024 * 1024)
        .add_service(svc)
        .serve(addr)
        .await?;

    Ok(())
}
