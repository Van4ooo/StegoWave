use crate::services;
use crate::stego_wave_grpc::stego_wave_service_server::StegoWaveServiceServer;
use std::net::SocketAddr;
use stego_wave::configuration::StegoWaveLib;
use tonic::transport::Server;

const MAX_MESSAGE_SIZE: usize = 100 * 1024 * 1024;

pub fn run_server(
    addr: SocketAddr,
    settings: StegoWaveLib,
) -> impl Future<Output = Result<(), tonic::transport::Error>> {
    let stego_wave_service = services::StegoWaveServiceImpl::new(settings);
    let svc = StegoWaveServiceServer::new(stego_wave_service)
        .max_encoding_message_size(MAX_MESSAGE_SIZE)
        .max_decoding_message_size(MAX_MESSAGE_SIZE);

    Server::builder().add_service(svc).serve(addr)
}
