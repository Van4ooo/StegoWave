use crate::models::request_object::{ClearRequest, ExtractRequest, HideRequest};
use actix_multipart::{Field, Multipart};
use futures::{StreamExt, TryStreamExt};
use std::error::Error;
use std::mem;
use tracing::{debug, warn};

#[derive(Default)]
pub struct MultipartyPayload {
    pub file_bytes: Option<Vec<u8>>,
    pub message: Option<String>,
    pub password: Option<String>,
    pub format: Option<String>,
    pub lsb_deep: Option<u8>,
}

impl TryFrom<MultipartyPayload> for HideRequest {
    type Error = String;

    fn try_from(mut value: MultipartyPayload) -> Result<Self, Self::Error> {
        Ok(HideRequest {
            file: value.get_file_bytes()?,
            message: value.get_message()?,
            password: value.get_password()?,
            format: value.get_format()?,
            lsb_deep: value.get_lsb_deep()?,
        })
    }
}

impl TryFrom<MultipartyPayload> for ExtractRequest {
    type Error = String;
    fn try_from(mut value: MultipartyPayload) -> Result<Self, Self::Error> {
        Ok(ExtractRequest {
            file: value.get_file_bytes()?,
            password: value.get_password()?,
            format: value.get_format()?,
            lsb_deep: value.get_lsb_deep()?,
        })
    }
}

impl TryFrom<MultipartyPayload> for ClearRequest {
    type Error = String;
    fn try_from(mut value: MultipartyPayload) -> Result<Self, Self::Error> {
        Ok(ClearRequest {
            file: value.get_file_bytes()?,
            password: value.get_password()?,
            format: value.get_format()?,
            lsb_deep: value.get_lsb_deep()?,
        })
    }
}

impl MultipartyPayload {
    pub fn get_file_bytes(&mut self) -> Result<Vec<u8>, String> {
        if let Some(file) = mem::take(&mut self.file_bytes) {
            Ok(file)
        } else {
            Err("The |file bytes| field is required".to_string())
        }
    }

    pub fn get_message(&mut self) -> Result<String, String> {
        if let Some(message) = mem::take(&mut self.message) {
            Ok(message)
        } else {
            Err("The |message| field is required".to_string())
        }
    }

    pub fn get_password(&mut self) -> Result<String, String> {
        if let Some(password) = mem::take(&mut self.password) {
            Ok(password)
        } else {
            Err("The |password| field is required".to_string())
        }
    }

    pub fn get_format(&mut self) -> Result<String, String> {
        if let Some(format) = mem::take(&mut self.format) {
            Ok(format)
        } else {
            Err("The |format| field is required".to_string())
        }
    }

    pub fn get_lsb_deep(&mut self) -> Result<u8, String> {
        if let Some(lsb_deep) = mem::take(&mut self.lsb_deep) {
            Ok(lsb_deep)
        } else {
            Err("The |lsb_deep| field is required".to_string())
        }
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
