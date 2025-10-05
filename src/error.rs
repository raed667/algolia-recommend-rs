use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Algolia API error (status {status}): {message:?}")]
    Api {
        status: u16,
        message: Option<String>,
        body: String,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
