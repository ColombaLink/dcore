use std::str::Utf8Error;
use thiserror::Error;
#[derive(Debug, Error)]
pub enum Error {
    #[error("internal I/O error")]
    IO(#[from] std::io::Error),

    #[error("`{0}`")]
    Other(String),

    #[error("`{0}`")]
    Utf8Error(#[from]  Utf8Error),

    #[error("`{0}`")]
    GpgmeError(#[from]  gpgme::Error),

}
