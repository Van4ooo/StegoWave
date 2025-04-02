use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use thiserror::Error;

#[derive(Debug)]
pub enum StegoError {
    InvalidFile(String),
    NotEnoughSamples(usize),
    HoundError(hound::Error),
    IncorrectPassword,
    FailedToReceiveMessage,
    Other(String),
}

impl fmt::Display for StegoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            StegoError::InvalidFile(err) => write!(f, "InvalidFileError: {err}"),
            StegoError::HoundError(err) => write!(f, "{err}"),
            StegoError::NotEnoughSamples(require_bits) => {
                write!(f, "NotEnoughSamplesError: minimum required {require_bits}")
            }
            StegoError::IncorrectPassword => write!(f, "Error password is incorrect"),
            StegoError::FailedToReceiveMessage => {
                write!(f, "Could not receive message, file may be corrupted")
            }
            StegoError::Other(err) => write!(f, "{err}"),
        }
    }
}

impl Error for StegoError {}

impl From<hound::Error> for StegoError {
    fn from(value: hound::Error) -> Self {
        StegoError::HoundError(value)
    }
}

#[derive(Debug, Error)]
pub enum StegoWaveClientError {
    #[error("Connection failed")]
    ConnectionFailed,

    #[error("Request failed")]
    RequestFailed,

    #[error("Invalid URL")]
    UlrInvalid,

    #[error("{0}")]
    Response(String),
}

impl StegoWaveClientError {
    pub fn help_message(&self) -> Option<String> {
        match self {
            StegoWaveClientError::ConnectionFailed => {
                Some("Ensure the server is running and check your network connection.".to_string())
            }
            StegoWaveClientError::RequestFailed => Some(
                "Verify that the request parameters are correct and the server is available."
                    .to_string(),
            ),
            StegoWaveClientError::UlrInvalid => {
                Some("Check that the URL is in the correct format and reachable.".to_string())
            }
            StegoWaveClientError::Response(_) => None,
        }
    }
}
