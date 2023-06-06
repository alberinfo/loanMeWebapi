#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use argon2::{
    password_hash::{ rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString },
    Argon2
};

#[derive(sqlx::Type, serde::Deserialize, serde::Serialize, Debug, Default)]
#[sqlx(type_name = "tiposusuario", rename_all = "lowercase")]
pub enum TipoUsuario {
    #[default]
    Prestatario,
    Prestamista,
    Administrador
}

#[derive(sqlx::FromRow, serde::Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Usuario {
    #[serde(skip_deserializing)]
    pub id: i32,

    #[serde(default)]
    pub email: String,

    #[serde(default)]
    pub nombrecompleto: String,

    pub nombreusuario: String,

    pub contrasenna: String,

    pub idwallet: Option<String>,

    pub tipousuario: Option<TipoUsuario>
}

impl Usuario {
    pub async fn generatePwd(&self) -> String {
        let pwd = self.contrasenna.clone();
        let res = tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            return Argon2::default().hash_password(pwd.as_bytes(), &salt).unwrap().to_string();
        }).await;
        
        return res.unwrap();
    }

    pub async fn validatePwd(&self, PHC: String) -> bool {
        let pwd = self.contrasenna.clone();
        let res = tokio::task::spawn_blocking(move || {
            let parsed_hash = PasswordHash::new(&PHC).unwrap();
            return Argon2::default().verify_password(pwd.as_bytes(), &parsed_hash).is_ok();
        }).await;
    
        return res.unwrap();
    }

    pub async fn buscarUsuario(&self, dbPool: &sqlx::PgPool) -> sqlx::Result<Usuario> {
        let usuario: Result<Usuario, sqlx::Error>  = sqlx::query_as::<_, Usuario>("SELECT * FROM usuario WHERE nombreusuario = $1")
            .bind(&self.nombreusuario)
            .fetch_one(dbPool)
            .await;
    
        return usuario;
    }
    
    pub async fn guardarUsuario(&self, dbPool: &sqlx::PgPool) -> sqlx::Result<sqlx::postgres::PgQueryResult> {
        let res = sqlx::query("INSERT INTO usuario(email, nombrecompleto, nombreusuario, contrasenna, idwallet, tipousuario) VALUES($1, $2, $3, $4, $5, $6)")
            .bind(&self.email)
            .bind(&self.nombrecompleto)
            .bind(&self.nombreusuario)
            .bind(&self.contrasenna)
            .bind(&self.idwallet)
            .bind(self.tipousuario.as_ref().unwrap() as &TipoUsuario)
            .execute(dbPool)
            .await;
        return res;
    }
}