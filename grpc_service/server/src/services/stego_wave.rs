use std::sync::Arc;
use stego_wave::AudioSteganography;
use stego_wave::configuration::StegoWaveLib;
use stego_wave::formats::get_stego_by_str;
use tonic::{Request, Response, Status};

use crate::stego_wave_grpc::{
    AudioResponse, ClearRequest, ExtractRequest, HideRequest, MessageResponse,
    stego_wave_service_server::StegoWaveService,
};

#[derive(Default)]
pub struct StegoWaveServiceImpl {
    settings: Arc<StegoWaveLib>,
}

impl StegoWaveServiceImpl {
    pub fn new(settings: StegoWaveLib) -> Self {
        Self {
            settings: Arc::new(settings),
        }
    }
}

macro_rules! get_stego {
    ($format:expr, $lsb_deep:expr, $settings:expr) => {
        match get_stego_by_str(&$format, $lsb_deep as _, (*$settings).clone()) {
            Ok(stego) => stego,
            Err(err) => return Err(Status::invalid_argument(err.to_string())),
        }
    };
}

#[tonic::async_trait]
impl StegoWaveService for StegoWaveServiceImpl {
    async fn hide_message(
        &self,
        request: Request<HideRequest>,
    ) -> Result<Response<AudioResponse>, Status> {
        let HideRequest {
            file,
            message,
            password,
            format,
            lsb_deep,
        } = request.into_inner();
        let stego = get_stego!(format, lsb_deep, self.settings);

        let (mut samples, spec) = stego
            .read_samples_from_byte(file)
            .map_err(|err| Status::internal(err.to_string()))?;

        stego
            .hide_message_binary(&mut samples, &message, &password)
            .map_err(|err| Status::internal(err.to_string()))?;

        let output_byte = stego
            .write_samples_to_byte(spec, &samples)
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(AudioResponse { file: output_byte }))
    }

    async fn extract_message(
        &self,
        request: Request<ExtractRequest>,
    ) -> Result<Response<MessageResponse>, Status> {
        let ExtractRequest {
            file,
            password,
            format,
            lsb_deep,
        } = request.into_inner();
        let stego = get_stego!(format, lsb_deep, self.settings);

        let (samples, _spec) = stego
            .read_samples_from_byte(file)
            .map_err(|err| Status::internal(err.to_string()))?;

        let message = stego
            .extract_message_binary(&samples, &password)
            .map_err(|err| Status::internal(err.to_string()))?;

        let reply = MessageResponse { message };
        Ok(Response::new(reply))
    }

    async fn clear_message(
        &self,
        request: Request<ClearRequest>,
    ) -> Result<Response<AudioResponse>, Status> {
        let ClearRequest {
            file,
            password,
            format,
            lsb_deep,
        } = request.into_inner();
        let stego = get_stego!(format, lsb_deep, self.settings);

        let (mut samples, spec) = stego
            .read_samples_from_byte(file)
            .map_err(|err| Status::internal(err.to_string()))?;

        stego
            .clear_secret_message_binary(&mut samples, &password)
            .map_err(|err| Status::internal(err.to_string()))?;

        let output_byte = stego
            .write_samples_to_byte(spec, &samples)
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(AudioResponse { file: output_byte }))
    }
}
