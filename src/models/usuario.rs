#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use argon2::{
    password_hash::{ rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString },
    Argon2
};

use sqlx::{Row, Column};

#[derive(thiserror::Error, Debug)]
pub enum UserError {
    #[error("Thread failed to execute task until completion")]
    MultithreadError(#[from] tokio::task::JoinError),

    #[error("There was an error while executing a database query")]
    DbError(#[from] sqlx::Error)
}

#[derive(sqlx::Type, serde::Deserialize, serde::Serialize, Debug, Default, PartialEq, Clone)]
#[sqlx(type_name = "tiposusuario", rename_all = "lowercase")]
pub enum TipoUsuario {
    #[default]
    Prestatario,
    Prestamista,
    Administrador
}

#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Usuario {
    #[serde(skip)]
    pub id: i64,

    #[serde(default)]
    pub email: String,

    #[serde(default)]
    pub nombrecompleto: String,

    pub nombreusuario: String,

    pub contrasenna: String,

    pub idwallet: Option<String>,

    pub tipousuario: Option<TipoUsuario>,

    #[serde(skip)]
    pub habilitado: bool
}

impl Usuario {
    pub async fn generatePwd(&mut self) -> Result<String, UserError> {
        let pwd = self.contrasenna.clone();
        let res = tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            return Argon2::default().hash_password(pwd.as_bytes(), &salt).unwrap().to_string();
        }).await?;

        self.contrasenna = res.clone();
        return Ok(res);
    }

    pub async fn validatePwd(&self, PHC: String) -> Result<bool, UserError> {
        let pwd = self.contrasenna.clone();
        let res = tokio::task::spawn_blocking(move || {
            let parsed_hash = PasswordHash::new(&PHC).unwrap();
            return Argon2::default().verify_password(pwd.as_bytes(), &parsed_hash).is_ok();
        }).await?;
    
        return Ok(res);
    }

    pub async fn buscarUsuario(nombreusuario: &String, dbPool: &sqlx::PgPool) -> Result<Usuario, UserError> {
        let usuario: Usuario = sqlx::query_as::<_, Usuario>("SELECT * FROM usuario WHERE nombreusuario = $1")
            .bind(nombreusuario)
            .fetch_one(dbPool)
            .await?;
    
        return Ok(usuario);
    }

    pub async fn buscarUsuarioById(id: &String, dbPool: &sqlx::PgPool) -> Result<Usuario, UserError> {
        let usuario: Usuario = sqlx::query_as::<_, Usuario>("SELECT * FROM usuario WHERE id = $1")
            .bind(id)
            .fetch_one(dbPool)
            .await?;

        return Ok(usuario);
    }
    
    pub async fn getUserId(nombreusuario: &String, dbPool: &sqlx::PgPool) -> Result<i64, UserError> {
        let row = sqlx::query("SELECT ID FROM usuario WHERE nombreusuario = $1")
            .bind(nombreusuario)
            .fetch_one(dbPool)
            .await?;

        let col = row.column(0);
        return Ok(row.try_get::<i64, usize>(col.ordinal())?);
    }

    pub async fn crearUsuario(&self, dbPool: &sqlx::PgPool) -> Result<sqlx::postgres::PgQueryResult, UserError> {
        let res = sqlx::query("INSERT INTO usuario(email, nombrecompleto, nombreusuario, contrasenna, idwallet, tipousuario, habilitado) VALUES($1, $2, $3, $4, $5, $6, $7)")
            .bind(&self.email)
            .bind(&self.nombrecompleto)
            .bind(&self.nombreusuario)
            .bind(&self.contrasenna)
            .bind(&self.idwallet)
            .bind(self.tipousuario.as_ref().unwrap() as &TipoUsuario)
            .bind(false)
            .execute(dbPool)
            .await?;
        return Ok(res);
    }

    pub async fn actualizarUsuario(&self, dbPool: &sqlx::PgPool) -> Result<sqlx::postgres::PgQueryResult, UserError> {
        let usrId = Usuario::getUserId(&self.nombreusuario, dbPool).await?;

        let res = sqlx::query("UPDATE usuario SET email = $1, nombreUsuario = $2, contrasenna = $3, idWallet = $4 WHERE ID = $5")
            .bind(&self.email)
            .bind(&self.nombreusuario)
            .bind(&self.contrasenna)
            .bind(&self.idwallet)
            .bind(usrId)
            .execute(dbPool)
            .await?;
        return Ok(res);
    }

    pub async fn eliminarUsuario(&self, dbPool: &sqlx::PgPool) -> Result<sqlx::postgres::PgQueryResult, UserError> {
        let res = sqlx::query("DELETE FROM usuario WHERE nombreusuario = $1")
            .bind(&self.nombreusuario)
            .execute(dbPool)
            .await?;
        return Ok(res);
    }

    pub async fn habilitarUsuario(&self, dbPool: &sqlx::PgPool) -> Result<sqlx::postgres::PgQueryResult, UserError> {
        let res = sqlx::query("UPDATE usuario SET habilitado = true WHERE nombreusuario = $1")
            .bind(&self.nombreusuario)
            .execute(dbPool)
            .await?;
        return Ok(res);
    }

    pub async fn deshabilitarUsuario(&self, dbPool: &sqlx::PgPool) -> Result<sqlx::postgres::PgQueryResult, UserError> {
        let res = sqlx::query("UPDATE usuario SET habilitado = false WHERE nombreusuario = $1")
            .bind(&self.nombreusuario)
            .execute(dbPool)
            .await?;
        return Ok(res);
    }
}