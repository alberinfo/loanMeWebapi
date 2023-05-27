#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use argon2::{
    password_hash::{ rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString },
    Argon2
};

#[derive(sqlx::Type, serde::Deserialize, serde::Serialize, Debug)]
#[sqlx(type_name = "tiposusuario", rename_all = "lowercase")]
pub enum TipoUsuario {
    Prestatario,
    Prestamista,
    Administrador
}

fn defaultTipoUsuario() -> TipoUsuario {
    return TipoUsuario::Prestatario; //No se si es lo correcto, pero por ahora funciona.
}


#[derive(sqlx::FromRow, serde::Deserialize, serde::Serialize, Debug)]
pub struct Usuario {
    #[serde(skip_serializing, skip_deserializing)]
    pub id: i32,

    #[serde(default)]
    pub email: String,

    #[serde(default)]
    pub nombrecompleto: String,

    pub nombreusuario: String,

    #[serde(skip_serializing)]
    pub hashcontrasenna: String,

    #[serde(skip_serializing)]
    pub idwallet: Option<String>,

    #[serde(default = "defaultTipoUsuario")]
    pub tipousuario: TipoUsuario
}

impl Usuario {
    pub async fn generatePwd(&self) -> String {
        let pwd = self.hashcontrasenna.clone();
        let res = tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            return Argon2::default().hash_password(pwd.as_bytes(), &salt).unwrap().to_string();
        }).await;
        
        return res.unwrap();
    }

    pub async fn validatePwd(&self, PHC: String) -> bool {
        let pwd = self.hashcontrasenna.clone();
        let res = tokio::task::spawn_blocking(move || {
            let parsed_hash = PasswordHash::new(&PHC).unwrap();
            return Argon2::default().verify_password(pwd.as_bytes(), &parsed_hash).is_ok();
        }).await;
    
        return res.unwrap();
    }
}