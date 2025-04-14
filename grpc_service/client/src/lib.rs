pub mod stego_wave_grpc {
    tonic::include_proto!("stego_wave");
}

mod grpc_client;

pub use grpc_client::StegoWaveGrpcClient;
