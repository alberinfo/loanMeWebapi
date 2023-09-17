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