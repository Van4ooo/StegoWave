pub mod configuration;
pub mod services;
pub mod startup;

pub mod stego_wave_grpc {
    tonic::include_proto!("stego_wave");
}
