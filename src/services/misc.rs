#![allow(clippy::needless_return)]

use std::vec;

use argon2::password_hash::rand_core::{RngCore, OsRng};
use sha2::{Sha512_224, Digest};

pub async fn generateRnd(bits: usize) -> Result<String, tokio::task::JoinError> {
    let rnd = tokio::task::spawn_blocking(move || {
        let mut buf = vec![0u8; bits / 8]; //generate 256 bits of entropy
        OsRng.fill_bytes(&mut buf);
    
        return format!("{:X}", Sha512_224::digest(buf));
    }).await?;

    return Ok(rnd);
}

pub fn deserializeNaiveDateTime<'de, D>(
    deserializer: D,
) -> Result<chrono::NaiveDateTime, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let format = "%Y-%m-%d %H:%M:%S";
    let s: &str = serde::Deserialize::deserialize(deserializer)?;
    chrono::NaiveDateTime::parse_from_str(s, format).map_err(serde::de::Error::custom)
}

pub fn serializeNaiveDateTime<S>(
    date: &chrono::NaiveDateTime,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let format = "%Y-%m-%d %H:%M:%S";
    let s = format!("{}", date.format(format));
    serializer.serialize_str(&s)
}