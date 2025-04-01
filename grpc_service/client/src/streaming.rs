use crate::stego_wave_grpc::{AudioResponse, ClearRequest, ExtractRequest, HideRequest};
use futures::StreamExt;
use stego_wave::error::StegoWaveClientError;
use tonic::Streaming;

impl HideRequest {
    pub fn create_by_chunk(chunk: &[u8]) -> Self {
        Self {
            file: chunk.to_owned(),
            message: "".to_string(),
            password: "".to_string(),
            format: "".to_string(),
            lsb_deep: 0,
        }
    }
}

impl ExtractRequest {
    pub fn create_by_chunk(chunk: &[u8]) -> ExtractRequest {
        Self {
            file: chunk.to_owned(),
            password: "".to_string(),
            format: "".to_string(),
            lsb_deep: 0,
        }
    }
}

impl ClearRequest {
    pub fn create_by_chunk(chunk: &[u8]) -> ClearRequest {
        Self {
            file: chunk.to_owned(),
            password: "".to_string(),
            format: "".to_string(),
            lsb_deep: 0,
        }
    }
}

#[inline]
pub async fn get_output_audio(
    mut response_stream: Streaming<AudioResponse>,
) -> Result<Vec<u8>, StegoWaveClientError> {
    let mut output = Vec::new();
    while let Some(chunk) = response_stream.next().await {
        let audio_response =
            chunk.map_err(|err| StegoWaveClientError::Response(err.message().to_string()))?;
        output.extend(audio_response.file);
    }

    Ok(output)
}
