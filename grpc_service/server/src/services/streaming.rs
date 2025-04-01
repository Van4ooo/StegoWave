use crate::stego_wave_grpc::{AudioResponse, ClearRequest, ExtractRequest, HideRequest};
use bytes::Bytes;
use std::mem;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;
use tonic::codegen::tokio_stream::StreamExt;

pub enum AudioMetadata {
    Hide {
        message: String,
        password: String,
        format: String,
        lsb_deep: u32,
    },
    General {
        password: String,
        format: String,
        lsb_deep: u32,
    },
}

pub trait AudioRequestExt {
    fn get_file(&self) -> &[u8];
    fn get_metadata(&mut self) -> AudioMetadata;
}

impl AudioRequestExt for HideRequest {
    fn get_file(&self) -> &[u8] {
        &self.file
    }
    fn get_metadata(&mut self) -> AudioMetadata {
        AudioMetadata::Hide {
            message: mem::take(&mut self.message),
            password: mem::take(&mut self.password),
            format: mem::take(&mut self.format),
            lsb_deep: self.lsb_deep,
        }
    }
}

impl AudioRequestExt for ExtractRequest {
    fn get_file(&self) -> &[u8] {
        &self.file
    }
    fn get_metadata(&mut self) -> AudioMetadata {
        AudioMetadata::General {
            password: mem::take(&mut self.password),
            format: mem::take(&mut self.format),
            lsb_deep: self.lsb_deep,
        }
    }
}

impl AudioRequestExt for ClearRequest {
    fn get_file(&self) -> &[u8] {
        &self.file
    }
    fn get_metadata(&mut self) -> AudioMetadata {
        AudioMetadata::General {
            password: mem::take(&mut self.password),
            format: mem::take(&mut self.format),
            lsb_deep: self.lsb_deep,
        }
    }
}

pub async fn aggregate_file_from_stream<T>(
    mut stream: tonic::Streaming<T>,
) -> Result<(Vec<u8>, AudioMetadata), Status>
where
    T: AudioRequestExt + Send + 'static,
{
    let mut file: Vec<u8> = Vec::new();
    let mut metadata: Option<AudioMetadata> = None;

    while let Some(chunk) = stream.next().await {
        let mut chunk = chunk?;
        if metadata.is_none() {
            metadata = Some(chunk.get_metadata());
        }
        file.extend_from_slice(chunk.get_file());
    }

    let metadata = metadata.ok_or(Status::invalid_argument(
        "Metadata not received: at least one packet with information is expected.",
    ))?;

    Ok((file, metadata))
}

pub fn stream_file_as_chunks(
    file: Vec<u8>,
    chunk_size: usize,
) -> impl futures::Stream<Item = Result<AudioResponse, Status>> {
    let full_bytes = Bytes::from(file);
    let total_len = full_bytes.len();
    let (tx, rx) = mpsc::channel(4);

    tokio::spawn(async move {
        let mut start = 0;
        while start < total_len {
            let end = std::cmp::min(start + chunk_size, total_len);
            let chunk = full_bytes.slice(start..end);

            let resp = AudioResponse {
                file: chunk.to_vec(),
            };
            if tx.send(Ok(resp)).await.is_err() {
                break;
            }
            start = end;
        }
    });

    ReceiverStream::new(rx)
}
