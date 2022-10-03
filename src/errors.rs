use std::error::Error;
use std::fmt::{Display, Formatter};

pub type Result<T> = core::result::Result<T, ProxyError>;

#[derive(Debug)]
pub enum ProxyError {
    Initialization,
}

impl Display for ProxyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ProxyError {}
