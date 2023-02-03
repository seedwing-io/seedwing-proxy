use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("ActixWeb error: {0}")]
    ActixWeb(#[from] actix_web::Error),
    #[error("Crates API error: {0}")]
    CratesError(#[from] crates_io_api::Error),
    #[error("FromUtf8Error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("Git2 error: {0}")]
    GitGit2Error(#[from] git2::Error),
    #[error("Payload error: {0}")]
    PayloadError(#[from] actix_web::error::PayloadError),
    #[error("Proxy init error")] // todo: remove?
    ProxyInitializationError,
    #[error("Send request error: {0}")]
    SendRequestError(#[from] awc::error::SendRequestError),
    #[error("IO error: {0}")]
    StdIoError(#[from] std::io::Error),
    #[error("Error: {message}")]
    Other { message: String },
}

impl actix_web::ResponseError for Error {}
