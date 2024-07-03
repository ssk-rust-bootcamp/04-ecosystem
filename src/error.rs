use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Param Error: {0}")]
    Param(#[from] std::num::ParseIntError),
    #[error("Serialize Error: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("Big Error: {0:?}")]
    BigError(Box<BigError>),
    #[error("Custom Error: {0}")]
    Custom(String),
}
#[allow(unused)]
#[derive(Debug)]
pub struct BigError {
    a: String,
    b: Vec<String>,
    c: [u8; 64],
    d: u64,
}
