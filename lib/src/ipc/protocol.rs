use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

use anyhow::Result;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Request {
    Ping,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Response {
    Pong,
}

pub fn send<T: Serialize>(mut stream: &UnixStream, serializable: T) -> Result<()> {
    let json = serde_json::to_string(&serializable)?;

    writeln!(stream, "{json}")?;

    Ok(())
}

pub fn read<T: DeserializeOwned>(mut stream: &UnixStream) -> Result<T> {
    let mut json = String::new();

    stream.read_to_string(&mut json)?;

    let deserializable: T = serde_json::from_str(&json)?;

    Ok(deserializable)
}
