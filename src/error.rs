use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("invalid HTTP header value: {0}")]
    InvalidHeader(#[from] reqwest::header::InvalidHeaderValue),

    #[error("invalid Runpod API URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("Runpod request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Runpod API returned {status}: {message}")]
    Api { status: StatusCode, message: String },
}

pub type Result<T> = std::result::Result<T, Error>;
