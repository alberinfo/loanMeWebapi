#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use argon2::password_hash::rand_core::{RngCore, OsRng};
use sha2::{Sha512, Digest};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Session {
    #[serde(skip_serializing)]
    pub username: String, //which user does this session belong to?
    #[serde(rename="sessionId")]
    pub id: String,
    pub creationDate: chrono::DateTime<chrono::Utc>
}

impl Session {
    pub async fn new(user: String) -> Session { //Create a new session for a given user
        let sessionIdHash = tokio::task::spawn_blocking(|| {
            let mut buf = [0u8; 32]; //generate 256 bits of entropy
            OsRng.fill_bytes(&mut buf);
    
            return format!("{:X}", Sha512::digest(buf));
        }).await.unwrap();

        let newSession = Session {
            username: user,
            id: sessionIdHash,
            creationDate: chrono::Utc::now()
        };

        return newSession;
    }
}