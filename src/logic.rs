use argon2::{
    password_hash::{ rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString },
    Argon2
};

pub async fn generatePwdPHC(pwd: String) -> String {
    let res = tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        return Argon2::default().hash_password(pwd.as_bytes(), &salt).unwrap().to_string();
    }).await;
    
    return res.unwrap();
}