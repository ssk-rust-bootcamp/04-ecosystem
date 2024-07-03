use anyhow::Context;
use std::{fs, mem::size_of};
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

fn main() -> Result<(), anyhow::Error> {
    println!("size of anyhow::Error:{}", size_of::<anyhow::Error>());
    println!("size of std::io:Error:{}", size_of::<std::io::Error>());
    println!(
        "size of std::num:ParseIntError:{}",
        size_of::<std::num::ParseIntError>()
    );

    println!("size of serde_json::Error:{}", size_of::<serde_json::Error>());
    println!("size of string is :{}", size_of::<String>());
    println!("size of MyError:{}", size_of::<MyError>());

    let filename = "non-existent-file.txt";
    let _fd = fs::File::open(filename).with_context(|| format!("Can not find file {}", filename))?;
    fail_with_error()?;
    Ok(())
}
fn fail_with_error() -> Result<(), MyError> {
    Err(MyError::Custom("This is a custom error".to_string()))
}
