use crate::services;
use crate::stego_wave_grpc::stego_wave_service_server::StegoWaveServiceServer;
use std::net::SocketAddr;
use stego_wave::configuration::StegoWaveLib;
use tonic::transport::Server;

pub fn run_server(
    addr: SocketAddr,
    settings: StegoWaveLib,
) -> impl Future<Output = Result<(), tonic::transport::Error>> {
    let stego_wave_service = services::StegoWaveServiceImpl::new(settings);
    let svc = StegoWaveServiceServer::new(stego_wave_service);

    Server::builder().add_service(svc).serve(addr)
}
