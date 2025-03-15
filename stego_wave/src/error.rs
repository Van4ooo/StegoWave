use std::error::Error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum StegoError {
    InvalidFile(String),
    NotEnoughSamples(usize),
    HoundError(hound::Error),
    IncorrectPassword,
    FailedToReceiveMessage,
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
        }
    }
}

impl Error for StegoError {}

impl From<hound::Error> for StegoError {
    fn from(value: hound::Error) -> Self {
        StegoError::HoundError(value)
    }
}
