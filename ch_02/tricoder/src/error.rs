use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Usage: tricoder <kerkour.com>")]
    CliUsage,
    #[error("Reqwest: {0}")]
    Reqwest(String),

    /// The certificate transparency source (crt.sh by default) is unavailable,
    /// returned an error status, or answered with something that is not JSON.
    /// crt.sh is a free public service with no availability guarantee, so this
    /// is an expected condition rather than a bug in the scanner.
    #[error("Upstream unavailable: {0}")]
    UpstreamUnavailable(String),
}

impl std::convert::From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Reqwest(err.to_string())
    }
}
