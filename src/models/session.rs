#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use argon2::password_hash::rand_core::{RngCore, OsRng};
use sha2::{Sha512, Digest};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Session {
    pub id: String,
    pub creationDate: String //Would probably need to change
}

impl Session {
    pub async fn new() -> Session {
        let sessionIdHash = tokio::task::spawn_blocking(|| {
            let mut buf = [0u8; 32]; //generate 256 bits of entropy
            OsRng.fill_bytes(&mut buf);
    
            return format!("{:X}", Sha512::digest(buf));
        }).await.unwrap();

        let newSession = Session {
            id: sessionIdHash,
            creationDate: "now".to_string()
        };

        return newSession;
    }
}