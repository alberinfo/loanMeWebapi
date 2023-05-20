use argon2::{
    password_hash::{ rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString },
    Argon2
};
use tokio::task::JoinError;

pub async fn generatePwdPHC(pwd: String) -> String {
    let res = tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        return Argon2::default().hash_password(pwd.as_bytes(), &salt).unwrap().to_string();
    }).await;
    
    return res.unwrap();
}

pub async fn validatePwdPHC(pwd: String, PHC: String) -> bool {
    let res = tokio::task::spawn_blocking(move || {
        let parsed_hash = PasswordHash::new(&PHC).unwrap();
        return Argon2::default().verify_password(pwd.as_bytes(), &parsed_hash).is_ok();
    }).await;

    return res.unwrap();
}