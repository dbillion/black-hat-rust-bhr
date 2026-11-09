use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Usage: tricoder <kerkour.com>")]
    CliUsage,
    #[error("Reqwest: {0}")]
    Reqwest(String),
    #[error("tokio join error: {0}")]
    TokioJoinError(String),
    /// An upstream service answered with a non-success status or a body that
    /// could not be decoded. Carries the module name and the detail so the
    /// failure is actionable rather than "Invalid HTTP response".
    #[error("{module}: upstream failed: {detail}")]
    InvalidHttpResponse { module: String, detail: String },
}

impl std::convert::From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Reqwest(err.to_string())
    }
}

impl std::convert::From<tokio::task::JoinError> for Error {
    fn from(err: tokio::task::JoinError) -> Self {
        Error::TokioJoinError(err.to_string())
    }
}
