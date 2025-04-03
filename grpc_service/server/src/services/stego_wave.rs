use crate::services::streaming::{
    AudioMetadata, aggregate_file_from_stream, stream_file_as_chunks,
};
use std::pin::Pin;
use std::sync::Arc;
use stego_wave::AudioSteganography;
use stego_wave::configuration::StegoWaveLib;
use stego_wave::formats::get_stego_by_str;
use tonic::codegen::tokio_stream::Stream;
use tonic::{Request, Response, Status};

use crate::stego_wave_grpc::{
    AudioResponse, ClearRequest, ExtractRequest, HideRequest, MessageResponse,
    stego_wave_service_server::StegoWaveService,
};

const CHUNK_SIZE: usize = 1024 * 1024;

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

#[tonic::async_trait]
impl StegoWaveService for StegoWaveServiceImpl {
    type HideMessageStream = Pin<Box<dyn Stream<Item = Result<AudioResponse, Status>> + Send>>;

    async fn hide_message(
        &self,
        request: Request<tonic::Streaming<HideRequest>>,
    ) -> Result<Response<Self::HideMessageStream>, Status> {
        let (file, metadata) = aggregate_file_from_stream(request.into_inner()).await?;

        let (message, password, format, lsb_deep) = match metadata {
            AudioMetadata::Hide {
                message,
                password,
                format,
                lsb_deep,
            } => (message, password, format, lsb_deep),
            AudioMetadata::General { .. } => {
                return Err(Status::internal("Invalid metadata format"));
            }
        };

        let stego = match get_stego_by_str(&format, lsb_deep as _, (*self.settings).clone()) {
            Ok(stego) => stego,
            Err(err) => return Err(Status::invalid_argument(err.to_string())),
        };

        let (mut samples, spec) = stego
            .read_samples_from_byte(file)
            .map_err(|err| Status::internal(err.to_string()))?;

        stego
            .hide_message_binary(&mut samples, &message, &password)
            .map_err(|err| Status::internal(err.to_string()))?;

        let output_byte = stego
            .write_samples_to_byte(spec, &samples)
            .map_err(|err| Status::internal(err.to_string()))?;

        let stream: Self::HideMessageStream =
            Box::pin(stream_file_as_chunks(output_byte, CHUNK_SIZE));
        Ok(Response::new(stream))
    }

    async fn extract_message(
        &self,
        request: Request<tonic::Streaming<ExtractRequest>>,
    ) -> Result<Response<MessageResponse>, Status> {
        let (file, metadata) = aggregate_file_from_stream(request.into_inner()).await?;

        let (password, format, lsb_deep) = match metadata {
            AudioMetadata::Hide { .. } => {
                return Err(Status::internal("Invalid metadata format"));
            }
            AudioMetadata::General {
                password,
                format,
                lsb_deep,
            } => (password, format, lsb_deep),
        };

        let stego = match get_stego_by_str(&format, lsb_deep as _, (*self.settings).clone()) {
            Ok(stego) => stego,
            Err(err) => return Err(Status::invalid_argument(err.to_string())),
        };

        let (samples, _spec) = stego
            .read_samples_from_byte(file)
            .map_err(|err| Status::internal(err.to_string()))?;

        let message = stego
            .extract_message_binary(&samples, &password)
            .map_err(|err| Status::internal(err.to_string()))?;

        let reply = MessageResponse { message };
        Ok(Response::new(reply))
    }

    type ClearMessageStream = Pin<Box<dyn Stream<Item = Result<AudioResponse, Status>> + Send>>;

    async fn clear_message(
        &self,
        request: Request<tonic::Streaming<ClearRequest>>,
    ) -> Result<Response<Self::ClearMessageStream>, Status> {
        let (file, metadata) = aggregate_file_from_stream(request.into_inner()).await?;

        let (password, format, lsb_deep) = match metadata {
            AudioMetadata::Hide { .. } => {
                return Err(Status::internal("Invalid metadata format"));
            }
            AudioMetadata::General {
                password,
                format,
                lsb_deep,
            } => (password, format, lsb_deep),
        };

        let stego = match get_stego_by_str(&format, lsb_deep as _, (*self.settings).clone()) {
            Ok(stego) => stego,
            Err(err) => return Err(Status::invalid_argument(err.to_string())),
        };

        let (mut samples, spec) = stego
            .read_samples_from_byte(file)
            .map_err(|err| Status::internal(err.to_string()))?;

        stego
            .clear_secret_message_binary(&mut samples, &password)
            .map_err(|err| Status::internal(err.to_string()))?;

        let output_byte = stego
            .write_samples_to_byte(spec, &samples)
            .map_err(|err| Status::internal(err.to_string()))?;

        let stream: Self::ClearMessageStream =
            Box::pin(stream_file_as_chunks(output_byte, CHUNK_SIZE));
        Ok(Response::new(stream))
    }
}
