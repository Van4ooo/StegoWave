use reqwest::Client;
use reqwest::multipart::{Form, Part};
use stego_wave::error::StegoWaveClientError;
use url::Url;

use stego_wave::object::StegoWaveClient;

fn convert_reqwest_error(err: reqwest::Error) -> StegoWaveClientError {
    if err.is_connect() {
        StegoWaveClientError::ConnectionFailed
    } else {
        StegoWaveClientError::RequestFailed
    }
}

#[derive(Clone)]
pub struct StegoWaveRestClient {
    rest_url: Url,
    client: Client,
}

impl StegoWaveRestClient {
    pub async fn new(url: impl TryInto<Url> + Send) -> Result<Self, StegoWaveClientError> {
        let rest_url = url
            .try_into()
            .map_err(|_err| StegoWaveClientError::UlrInvalid)?;

        Ok(Self {
            rest_url,
            client: Client::new(),
        })
    }
}

#[async_trait::async_trait]
impl StegoWaveClient for StegoWaveRestClient {
    async fn hide_message(
        &mut self,
        file: Vec<u8>,
        file_name: String,
        message: String,
        password: String,
        format: String,
        lsb_deep: u8,
    ) -> Result<Vec<u8>, StegoWaveClientError> {
        let form = Form::new()
            .part("file", Part::bytes(file).file_name(file_name))
            .text("message", message)
            .text("password", password)
            .text("format", format)
            .text("lsb_deep", lsb_deep.to_string());

        let url = self
            .rest_url
            .join("api/hide_message")
            .map_err(|_err| StegoWaveClientError::UlrInvalid)?;

        let response = self
            .client
            .post(url)
            .multipart(form)
            .send()
            .await
            .map_err(convert_reqwest_error)?;

        if !response.status().is_success() {
            let err_text = response.text().await.map_err(convert_reqwest_error)?;
            return Err(StegoWaveClientError::Response(err_text));
        }

        let bytes = response.bytes().await.map_err(convert_reqwest_error)?;
        Ok(bytes.to_vec())
    }

    async fn extract_message(
        &mut self,
        file: Vec<u8>,
        file_name: String,
        password: String,
        format: String,
        lsb_deep: u8,
    ) -> Result<String, StegoWaveClientError> {
        let form = Form::new()
            .part("file", Part::bytes(file).file_name(file_name))
            .text("password", password)
            .text("format", format)
            .text("lsb_deep", lsb_deep.to_string());

        let url = self
            .rest_url
            .join("api/extract_message")
            .map_err(|_err| StegoWaveClientError::UlrInvalid)?;
        let response = self
            .client
            .post(url)
            .multipart(form)
            .send()
            .await
            .map_err(convert_reqwest_error)?;

        if !response.status().is_success() {
            let err_text = response.text().await.map_err(convert_reqwest_error)?;
            return Err(StegoWaveClientError::Response(err_text));
        }
        let secret_message = response.text().await.map_err(convert_reqwest_error)?;

        Ok(secret_message)
    }

    async fn clear_message(
        &mut self,
        file: Vec<u8>,
        file_name: String,
        password: String,
        format: String,
        lsb_deep: u8,
    ) -> Result<Vec<u8>, StegoWaveClientError> {
        let form = Form::new()
            .part("file", Part::bytes(file).file_name(file_name))
            .text("password", password)
            .text("format", format)
            .text("lsb_deep", lsb_deep.to_string());

        let url = self
            .rest_url
            .join("api/clear_message")
            .map_err(|_err| StegoWaveClientError::UlrInvalid)?;
        let response = self
            .client
            .post(url)
            .multipart(form)
            .send()
            .await
            .map_err(convert_reqwest_error)?;

        if !response.status().is_success() {
            let err_text = response.text().await.map_err(convert_reqwest_error)?;
            return Err(StegoWaveClientError::Response(err_text));
        }

        let bytes = response.bytes().await.map_err(convert_reqwest_error)?;
        Ok(bytes.to_vec())
    }
}
