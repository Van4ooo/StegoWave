use stego_wave::AudioSteganography;
use stego_wave::formats::get_stego_by_str;
use tonic::{Request, Response, Status};

use crate::stego_wave_grpc::{
    AudioResponse, ClearRequest, ExtractRequest, HideRequest, MessageResponse,
    stego_wave_service_server::StegoWaveService,
};

#[derive(Default)]
pub struct StegoWaveServiceImpl {}

#[tonic::async_trait]
impl StegoWaveService for StegoWaveServiceImpl {
    async fn hide_message(
        &self,
        request: Request<HideRequest>,
    ) -> Result<Response<AudioResponse>, Status> {
        let req = request.into_inner();
        let stego = match get_stego_by_str(&req.format, req.lsb_deep as _) {
            Ok(stego) => stego,
            Err(err) => return Err(Status::invalid_argument(err)),
        };

        let (mut samples, spec) = stego
            .read_samples_from_byte(req.file)
            .map_err(|err| Status::internal(err.to_string()))?;

        stego
            .hide_message_binary(&mut samples, &req.message, &req.password)
            .map_err(|err| Status::internal(err.to_string()))?;

        let output_byte = stego
            .write_samples_to_byte(spec, &samples)
            .map_err(|err| Status::internal(err.to_string()))?;

        let reply = AudioResponse { file: output_byte };
        Ok(Response::new(reply))
    }

    async fn extract_message(
        &self,
        request: Request<ExtractRequest>,
    ) -> Result<Response<MessageResponse>, Status> {
        let req = request.into_inner();
        let stego = match get_stego_by_str(&req.format, req.lsb_deep as _) {
            Ok(stego) => stego,
            Err(err) => return Err(Status::invalid_argument(err)),
        };

        let (samples, _spec) = stego
            .read_samples_from_byte(req.file)
            .map_err(|err| Status::internal(err.to_string()))?;

        let message = stego
            .extract_message_binary(&samples, &req.password)
            .map_err(|err| Status::internal(err.to_string()))?;

        let reply = MessageResponse { message };
        Ok(Response::new(reply))
    }

    async fn clear_message(
        &self,
        request: Request<ClearRequest>,
    ) -> Result<Response<AudioResponse>, Status> {
        let req = request.into_inner();
        let stego = match get_stego_by_str(&req.format, req.lsb_deep as _) {
            Ok(stego) => stego,
            Err(err) => return Err(Status::invalid_argument(err)),
        };

        let (mut samples, spec) = stego
            .read_samples_from_byte(req.file)
            .map_err(|err| Status::internal(err.to_string()))?;

        stego
            .clear_secret_message_binary(&mut samples, &req.password)
            .map_err(|err| Status::internal(err.to_string()))?;

        let output_byte = stego
            .write_samples_to_byte(spec, &samples)
            .map_err(|err| Status::internal(err.to_string()))?;

        let reply = AudioResponse { file: output_byte };
        Ok(Response::new(reply))
    }
}
