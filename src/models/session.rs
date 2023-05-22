#![allow(non_snake_case)]

use argon2::password_hash::rand_core::{CryptoRngCore, OsRng};
use sha2::{Sha512, Digest};

pub struct session {
    id: String,
    creationDate: String //Would probably need to change
}

impl session {
    pub fn new(mut rng: impl CryptoRngCore) -> session {
        let mut buf = [0u8; 8];
        rng.fill_bytes(&mut buf);

        let randNum = u64::from_be_bytes(buf);
        let sessionIdHash = format!("{:X}", Sha512::digest(randNum.to_string()));

        println!("{}", sessionIdHash);

        let newSession = session {
            id: sessionIdHash,
            creationDate: "now".to_string()
        };

        return newSession;
    }
}