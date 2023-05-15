//use sqlx::{mssql::MssqlQueryResult};
//use sqlx::postgres;
use serde::{Serialize, Deserialize};

#[derive(sqlx::FromRow, serde::Deserialize, serde::Serialize, Debug)]
pub struct Usuario {
    #[serde(skip_serializing, skip_deserializing)]
    pub id: i32,
    pub email: String,
    pub nombrecompleto: String,
    pub nombreusuario: String,
    pub hashcontrasenna: String,
    pub salt: String,
    pub idwallet: Option<String>,
    pub dni: String
}

pub async fn getDbConnection() -> sqlx::PgPool {
    return sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap()).await.unwrap();
}

pub async fn getTableCount(dbPool: &sqlx::PgPool) -> String {
    return sqlx::query("SELECT COUNT(*) from information_schema.tables where table_schema = 'public'").fetch_one(dbPool).await.unwrap();
}

pub async fn buscarUsuario(nomUsuario: String, dbPool: &sqlx::PgPool) -> sqlx::Result<Usuario> {
    let usuario: Result<Usuario, sqlx::Error>  = sqlx::query_as::<_, Usuario>(r#"SELECT * FROM prestatario WHERE nombreusuario = $1"#)
        .bind(nomUsuario)
        .fetch_one(dbPool)
        .await;

    return usuario;
}