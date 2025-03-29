use crate::stego_wave_grpc::stego_wave_service_client::StegoWaveServiceClient;
use crate::stego_wave_grpc::{ClearRequest, ExtractRequest, HideRequest};
use crate::streaming::get_output_audio;
use stego_wave::error::StegoWaveClientError;
use stego_wave::object::StegoWaveClient;
use tonic::Request;
use tonic::transport::Channel;
use url::Url;

const CHUNK_SIZE: usize = 1024 * 1024;

#[derive(Clone)]
pub struct StegoWaveGrpcClient {
    client: StegoWaveServiceClient<Channel>,
}

fn create_chunks(file: &[u8], chunk_size: usize) -> impl Iterator<Item = &[u8]> {
    file.chunks(chunk_size)
}

impl StegoWaveGrpcClient {
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
impl StegoWaveClient for StegoWaveGrpcClient {
    async fn hide_message(
        &mut self,
        file: Vec<u8>,
        _file_name: String,
        message: String,
        password: String,
        format: String,
        lsb_deep: u8,
    ) -> Result<Vec<u8>, StegoWaveClientError> {
        let mut first_chunk = true;
        let request_stream = tokio_stream::iter(
            create_chunks(&file, CHUNK_SIZE)
                .map(|chunk| {
                    if first_chunk {
                        first_chunk = false;
                        HideRequest {
                            file: chunk.to_owned(),
                            message: message.clone(),
                            password: password.clone(),
                            format: format.clone(),
                            lsb_deep: lsb_deep as u32,
                        }
                    } else {
                        HideRequest::create_by_chunk(chunk)
                    }
                })
                .collect::<Vec<_>>(),
        );

        let response_stream = self
            .client
            .hide_message(Request::new(request_stream))
            .await
            .map_err(|err| StegoWaveClientError::Response(err.message().to_string()))?
            .into_inner();

        get_output_audio(response_stream).await
    }

    async fn extract_message(
        &mut self,
        file: Vec<u8>,
        _file_name: String,
        password: String,
        format: String,
        lsb_deep: u8,
    ) -> Result<String, StegoWaveClientError> {
        let mut first_chunk = true;
        let request_stream = tokio_stream::iter(
            create_chunks(&file, CHUNK_SIZE)
                .map(|chunk| {
                    if first_chunk {
                        first_chunk = false;
                        ExtractRequest {
                            file: chunk.to_owned(),
                            password: password.clone(),
                            format: format.clone(),
                            lsb_deep: lsb_deep as u32,
                        }
                    } else {
                        ExtractRequest::create_by_chunk(chunk)
                    }
                })
                .collect::<Vec<_>>(),
        );

        let response = self
            .client
            .extract_message(Request::new(request_stream))
            .await
            .map_err(|err| StegoWaveClientError::Response(err.message().to_string()))?
            .into_inner();

        Ok(response.message)
    }

    async fn clear_message(
        &mut self,
        file: Vec<u8>,
        _file_name: String,
        password: String,
        format: String,
        lsb_deep: u8,
    ) -> Result<Vec<u8>, StegoWaveClientError> {
        let mut first_chunk = true;
        let request_stream = tokio_stream::iter(
            create_chunks(&file, CHUNK_SIZE)
                .map(|chunk| {
                    if first_chunk {
                        first_chunk = false;
                        ClearRequest {
                            file: chunk.to_owned(),
                            password: password.clone(),
                            format: format.clone(),
                            lsb_deep: lsb_deep as u32,
                        }
                    } else {
                        ClearRequest::create_by_chunk(chunk)
                    }
                })
                .collect::<Vec<_>>(),
        );

        let response_stream = self
            .client
            .clear_message(Request::new(request_stream))
            .await
            .map_err(|err| StegoWaveClientError::Response(err.message().to_string()))?
            .into_inner();

        get_output_audio(response_stream).await
    }
}
