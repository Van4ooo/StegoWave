use actix_multipart::{Field, Multipart};
use futures::{StreamExt, TryStreamExt};
use std::error::Error;
use std::mem;
use stego_wave::AudioSteganography;
use stego_wave::formats::wav::WAV16;
use tracing::{debug, warn};

#[derive(Default)]
pub struct MultipartyPayload {
    pub file_bytes: Option<Vec<u8>>,
    pub message: Option<String>,
    pub password: Option<String>,
    pub format: Option<String>,
    pub lsb_deep: Option<u8>,
}

impl MultipartyPayload {
    pub fn get_required_field(
        &mut self,
        file_bytes: bool,
        message: bool,
        password: bool,
        format: bool,
    ) -> Result<(Vec<u8>, String, String, String, u8), String> {
        if file_bytes && self.file_bytes.is_none() {
            return Err("The |file body| field is required".to_string());
        }
        if message && self.message.is_none() {
            return Err("The |message| field is required".to_string());
        }
        if password && self.password.is_none() {
            return Err("The |password| field is required".to_string());
        }
        if format && self.format.is_none() {
            return Err("The |format| field is required".to_string());
        }

        let payload = mem::take(self);
        Ok((
            payload.file_bytes.unwrap_or_default(),
            payload.message.unwrap_or_default(),
            payload.password.unwrap_or_default(),
            payload.format.unwrap_or_default(),
            payload.lsb_deep.unwrap_or(1),
        ))
    }
}

pub async fn parse_multipart_payload(mut payload: Multipart) -> Result<MultipartyPayload, String> {
    let mut file_bytes = None;
    let mut message = None;
    let mut password = None;
    let mut format = None;
    let mut lsb_deep = None;

    while let Ok(Some(field)) = payload.try_next().await {
        let name = field
            .content_disposition()
            .and_then(|cd| cd.get_name())
            .unwrap_or_default();

        match name {
            "file" => {
                let data = get_byte_from_field(field).await.map_err(|err| {
                    warn!("Failed to get |file| :: {err}");
                    "Failed to get |file|"
                })?;
                file_bytes = Some(data);
            }
            "message" => {
                let text = get_text_from_field(field).await.map_err(|err| {
                    warn!("Failed to get |message| :: {err}");
                    format!("Failed to get |message| :: {err}")
                })?;
                message = Some(text);
            }
            "password" => {
                let text = get_text_from_field(field).await.map_err(|err| {
                    warn!("Failed to get |password| :: {err}");
                    format!("Failed to get |password| :: {err}")
                })?;
                password = Some(text);
            }
            "format" => {
                let text = get_text_from_field(field).await.map_err(|err| {
                    warn!("Failed to get |format| :: {err}");
                    format!("Failed to get |format| :: {err}")
                })?;
                format = Some(text);
            }
            "lsb_deep" => {
                let text = get_text_from_field(field).await.map_err(|err| {
                    warn!("Failed to get |lsb_deep| :: {err}");
                    format!("Failed to get |lsb_deep| :: {err}")
                })?;
                lsb_deep = Some(text.parse::<u8>().unwrap_or(1));
            }
            _ => {}
        }
    }
    debug!(
        "MultipartyPayload :: format -> {}",
        format.clone().unwrap_or_default()
    );

    Ok(MultipartyPayload {
        file_bytes,
        message,
        password,
        format,
        lsb_deep,
    })
}

async fn get_byte_from_field(mut field: Field) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = Vec::new();
    while let Some(chunk) = field.next().await {
        let data = chunk?;
        bytes.extend_from_slice(&data);
    }
    Ok(bytes)
}

async fn get_text_from_field(mut field: Field) -> Result<String, Box<dyn Error>> {
    let mut bytes = Vec::new();
    while let Some(chunk) = field.next().await {
        let data = chunk?;
        bytes.extend_from_slice(&data);
    }
    Ok(String::from_utf8(bytes).unwrap_or_default())
}

pub fn get_format_instance(
    format: &str,
    lsb_deep: u8,
) -> Result<impl AudioSteganography<i16>, String> {
    match format {
        "wav16" => match WAV16::builder().lsb_deep(lsb_deep).build() {
            Ok(wav16) => Ok(wav16),
            Err(err) => Err(format!("{err}")),
        },
        _ => Err("Invalid format".to_string()),
    }
}
