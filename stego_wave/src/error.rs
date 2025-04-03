use thiserror::Error;

#[derive(Error, Debug)]
pub enum StegoError {
    #[error("InvalidFileError: {0}")]
    InvalidFile(String),

    #[error("NotEnoughSamplesError: minimum required {0}")]
    NotEnoughSamples(usize),

    #[error("{0}")]
    HoundError(#[from] hound::Error),

    #[error("Error password is incorrect")]
    IncorrectPassword,

    #[error("Could not receive message, file may be corrupted")]
    FailedToReceiveMessage,

    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Error)]
pub enum StegoWaveClientError {
    #[error("Connection failed")]
    ConnectionFailed,

    #[error("Request failed")]
    RequestFailed,

    #[error("Invalid server URL")]
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
