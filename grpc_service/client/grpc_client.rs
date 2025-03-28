use crate::stego_wave_grpc::stego_wave_service_client::StegoWaveServiceClient;
use crate::stego_wave_grpc::{ClearRequest, ExtractRequest, HideRequest};
use stego_wave::error::StegoWaveClientError;
use stego_wave::object::StegoWaveClient;
use tonic::transport::Channel;
use url::Url;

#[derive(Clone)]
pub struct StegoWaveGrpcClient {
    client: StegoWaveServiceClient<Channel>,
}

impl StegoWaveGrpcClient{
    pub async fn new(url: impl TryInto<Url> + Send) -> Result<Self, StegoWaveClientError> {
        let rest_url = url
            .try_into()
            .map_err(|_err| StegoWaveClientError::UlrInvalid)?;

        let url_owned = rest_url.to_string();
        let static_url: &'static str = Box::leak(url_owned.into_boxed_str());

        let client = StegoWaveServiceClient::connect(static_url)
            .await
            .map_err(|_err| StegoWaveClientError::ConnectionFailed)?;

        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl<'a> StegoWaveClient for StegoWaveGrpcClient {
    async fn hide_message(
        &mut self,
        file: Vec<u8>,
        _file_name: String,
        message: String,
        password: String,
        format: String,
        lsb_deep: u8,
    ) -> Result<Vec<u8>, StegoWaveClientError> {
        match self
            .client
            .hide_message(HideRequest {
                file,
                message,
                password,
                format,
                lsb_deep: lsb_deep as _,
            })
            .await
        {
            Ok(file_out) => Ok(file_out.into_inner().file),
            Err(err) => Err(StegoWaveClientError::Response(err.message().to_string())),
        }
    }

    async fn extract_message(
        &mut self,
        file: Vec<u8>,
        _file_name: String,
        password: String,
        format: String,
        lsb_deep: u8,
    ) -> Result<String, StegoWaveClientError> {
        match self
            .client
            .extract_message(ExtractRequest {
                file,
                password,
                format,
                lsb_deep: lsb_deep as _,
            })
            .await
        {
            Ok(secret_message) => Ok(secret_message.into_inner().message),
            Err(err) => Err(StegoWaveClientError::Response(err.message().to_string())),
        }
    }

    async fn clear_message(
        &mut self,
        file: Vec<u8>,
        _file_name: String,
        password: String,
        format: String,
        lsb_deep: u8,
    ) -> Result<Vec<u8>, StegoWaveClientError> {
        match self
            .client
            .clear_message(ClearRequest {
                file,
                password,
                format,
                lsb_deep: lsb_deep as _,
            })
            .await
        {
            Ok(file_out) => Ok(file_out.into_inner().file),
            Err(err) => Err(StegoWaveClientError::Response(err.message().to_string())),
        }
    }
}
