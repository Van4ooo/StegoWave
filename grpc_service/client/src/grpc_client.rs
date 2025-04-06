use crate::stego_wave_grpc::stego_wave_service_client::StegoWaveServiceClient;
use crate::stego_wave_grpc::{ClearRequest, ExtractRequest, HideRequest};
use stego_wave::error::StegoWaveClientError;
use stego_wave::object::StegoWaveClient;
use tonic::codegen::Bytes;
use tonic::transport::Channel;

const MAX_MESSAGE_SIZE: usize = 100 * 1024 * 1024;

#[derive(Clone)]
pub struct StegoWaveGrpcClient {
    client: StegoWaveServiceClient<Channel>,
}

impl StegoWaveGrpcClient {
    pub async fn new(url: impl Into<Bytes> + Send) -> Result<Self, StegoWaveClientError> {
        let channel = Channel::from_shared(url)
            .map_err(|err| StegoWaveClientError::UlrInvalid(err.to_string()))?
            .connect()
            .await
            .map_err(|_err| StegoWaveClientError::ConnectionFailed)?;

        let client = StegoWaveServiceClient::new(channel)
            .max_decoding_message_size(MAX_MESSAGE_SIZE)
            .max_encoding_message_size(MAX_MESSAGE_SIZE);

        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl StegoWaveClient for StegoWaveGrpcClient {
    async fn hide_message(
        &mut self,
        file: Vec<u8>,
        message: String,
        password: String,
        format: String,
        lsb_deep: u8,
    ) -> Result<Vec<u8>, StegoWaveClientError> {
        let response = self
            .client
            .hide_message(HideRequest {
                file,
                message,
                password,
                format,
                lsb_deep: lsb_deep as _,
            })
            .await
            .map_err(|err| StegoWaveClientError::Response(err.message().to_string()))?;

        Ok(response.into_inner().file)
    }

    async fn extract_message(
        &mut self,
        file: Vec<u8>,
        password: String,
        format: String,
        lsb_deep: u8,
    ) -> Result<String, StegoWaveClientError> {
        let response = self
            .client
            .extract_message(ExtractRequest {
                file,
                password,
                format,
                lsb_deep: lsb_deep as _,
            })
            .await
            .map_err(|err| StegoWaveClientError::Response(err.message().to_string()))?
            .into_inner();

        Ok(response.message)
    }

    async fn clear_message(
        &mut self,
        file: Vec<u8>,
        password: String,
        format: String,
        lsb_deep: u8,
    ) -> Result<Vec<u8>, StegoWaveClientError> {
        let response = self
            .client
            .clear_message(ClearRequest {
                file,
                password,
                format,
                lsb_deep: lsb_deep as _,
            })
            .await
            .map_err(|err| StegoWaveClientError::Response(err.message().to_string()))?;

        Ok(response.into_inner().file)
    }
}
