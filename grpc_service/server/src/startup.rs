use crate::configuration::Settings;
use crate::services;
use crate::stego_wave_grpc::stego_wave_service_server::StegoWaveServiceServer;
use std::net::SocketAddr;
use tonic::transport::Server;

pub fn run_server(
    addr: SocketAddr,
    settings: Settings,
) -> impl Future<Output = Result<(), tonic::transport::Error>> {
    let stego_wave_service = services::StegoWaveServiceImpl::new(settings.stego_wave_lib);
    let svc = StegoWaveServiceServer::new(stego_wave_service);

    Server::builder().add_service(svc).serve(addr)
}
