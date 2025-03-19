use crate::error::StegoWaveClientError;
use reqwest::Client;
use reqwest::multipart::{Form, Part};
use std::net::TcpStream;
use url::Url;

#[derive(Clone)]
pub struct StegoWaveRestClient {
    rest_url: Url,
    client: Client,
}

#[allow(unused)]
impl StegoWaveRestClient {
    pub fn new(url: impl Into<Url>) -> Self {
        Self {
            rest_url: url.into(),
            client: Client::new(),
        }
    }

    pub async fn hide_message(
        &self,
        file: Vec<u8>,
        file_name: impl Into<String>,
        message: impl Into<String>,
        password: impl Into<String>,
        format: impl Into<String>,
        lsb_deep: u8,
    ) -> Result<Vec<u8>, StegoWaveClientError> {
        let form = Form::new()
            .part("file", Part::bytes(file).file_name(file_name.into()))
            .text("message", message.into())
            .text("password", password.into())
            .text("format", format.into())
            .text("lsb_deep", lsb_deep.to_string());

        let url = self.rest_url.join("api/hide_message")?;
        let response = self.client.post(url).multipart(form).send().await?;

        if !response.status().is_success() {
            let err_text = response.text().await?;
            return Err(StegoWaveClientError::Response(err_text));
        }

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    pub async fn extract_message(
        &self,
        file: Vec<u8>,
        file_name: impl Into<String>,
        password: impl Into<String>,
        format: impl Into<String>,
        lsb_deep: u8,
    ) -> Result<String, StegoWaveClientError> {
        let form = Form::new()
            .part("file", Part::bytes(file).file_name(file_name.into()))
            .text("password", password.into())
            .text("format", format.into())
            .text("lsb_deep", lsb_deep.to_string());

        let url = self.rest_url.join("api/extract_message")?;
        let response = self.client.post(url).multipart(form).send().await?;

        if !response.status().is_success() {
            let err_text = response.text().await?;
            return Err(StegoWaveClientError::Response(err_text));
        }
        Ok(response.text().await?)
    }

    pub async fn clear_message(
        &self,
        file: Vec<u8>,
        file_name: impl Into<String>,
        password: impl Into<String>,
        format: impl Into<String>,
        lsb_deep: u8,
    ) -> Result<Vec<u8>, StegoWaveClientError> {
        let form = Form::new()
            .part("file", Part::bytes(file).file_name(file_name.into()))
            .text("password", password.into())
            .text("format", format.into())
            .text("lsb_deep", lsb_deep.to_string());

        let url = self.rest_url.join("api/clear_message")?;
        let response = self.client.post(url).multipart(form).send().await?;

        if !response.status().is_success() {
            let err_text = response.text().await?;
            return Err(StegoWaveClientError::Response(err_text));
        }

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    fn is_server_up(&self) -> bool {
        if let (Some(host), Some(port)) = (self.rest_url.host_str(), self.rest_url.port()) {
            TcpStream::connect(format!("{}:{}", host, port)).is_ok()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::error::StegoWaveClientError;
    use crate::rest_client::StegoWaveRestClient;
    use std::error::Error;
    use std::fs;
    use std::path::PathBuf;
    use url::Url;

    fn create_wav_file(
        path: &PathBuf,
        bits_per_sample: u16,
        samples: &[i16],
    ) -> Result<(), Box<dyn Error>> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 44_100,
            bits_per_sample,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create(path, spec)?;
        for &sample in samples {
            writer.write_sample(sample)?
        }

        writer.finalize()?;
        Ok(())
    }

    fn temp_path(filename: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(filename);
        path
    }

    macro_rules! create_client {
        ($url:expr) => {{
            let rest_url = Url::parse($url).unwrap();
            StegoWaveRestClient::new(rest_url)
        }};
    }

    macro_rules! run_if_server_up {
        ($body:block) => {{
            let client = create_client!("http://127.0.0.1:8080");

            if !client.is_server_up() {
                panic!("Server not available, skipping test_extract_message.");
            }

            $body
        }};
    }

    async fn test_extract_message(
        client: &StegoWaveRestClient,
        result: Vec<u8>,
        valid_password: bool,
    ) {
        for (password, lsb_deep, success) in [
            ("qwerty1234", 1, valid_password),
            ("qwerty1234", 2, false),
            ("password", 1, false),
        ] {
            match client
                .extract_message(
                    result.clone(),
                    "input_file.wav",
                    password,
                    "wav16",
                    lsb_deep,
                )
                .await
            {
                Ok(message) => {
                    assert!(success);
                    assert_eq!("Secret Message", &message);
                }
                Err(StegoWaveClientError::Response(err)) => {
                    assert!(!success);
                    assert_eq!("Error password is incorrect", &err)
                }
                _ => panic!(),
            }
        }
    }

    macro_rules! clear_message {
        ($client:expr, $result:expr, $password:expr) => {
            $client
                .clear_message($result, "input_full.wav", $password, "wav16", 1)
                .await
        };
    }

    #[tokio::test]
    async fn test_rest_client() -> Result<(), Box<dyn Error>> {
        run_if_server_up!({
            let client = create_client!("http://127.0.0.1:8080");

            let samples: Vec<i16> = vec![0; 10_000];
            let input_path = temp_path("input_full.wav");
            create_wav_file(&input_path, 16, &samples)?;

            let file_byte = fs::read(&input_path)?;
            let result = client
                .hide_message(
                    file_byte,
                    "input_full.wav",
                    "Secret Message",
                    "qwerty1234",
                    "wav16",
                    1,
                )
                .await?;

            test_extract_message(&client, result.clone(), true).await;
            assert!(clear_message!(&client, result.clone(), "qwerty").is_err());

            let result = clear_message!(&client, result.clone(), "qwerty1234")?;
            test_extract_message(&client, result, false).await;

            let _ = fs::remove_file(input_path);
        });

        Ok(())
    }
}
