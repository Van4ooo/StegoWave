use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum StegoWaveClientError {
    Request(reqwest::Error),
    Url(url::ParseError),
    Response(String),
}

impl fmt::Display for StegoWaveClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StegoWaveClientError::Request(e) => write!(f, "[!] Request: {}", e),
            StegoWaveClientError::Url(e) => write!(f, "[!] Url: {}", e),
            StegoWaveClientError::Response(msg) => write!(f, "[!] Response: {}", msg),
        }
    }
}

impl Error for StegoWaveClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            StegoWaveClientError::Request(e) => Some(e),
            StegoWaveClientError::Url(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for StegoWaveClientError {
    fn from(err: reqwest::Error) -> Self {
        StegoWaveClientError::Request(err)
    }
}

impl From<url::ParseError> for StegoWaveClientError {
    fn from(err: url::ParseError) -> Self {
        StegoWaveClientError::Url(err)
    }
}
