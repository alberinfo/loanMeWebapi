use sqlx::{postgres::{PgRow, PgQueryResult}, Row, Column};
use crate::models::usuario::{Usuario, TipoUsuario};

pub async fn getDbConnection() -> sqlx::PgPool {
    return sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap()).await.unwrap();
}

pub async fn getTableCount(dbPool: &sqlx::PgPool) -> i64 {
    let row: PgRow = sqlx::query("SELECT COUNT(*) from information_schema.tables where table_schema = 'public'").fetch_one(dbPool).await.unwrap();
    let col = row.column(0); //We know there is only one column for this query
    return row.try_get::<i64, usize>(col.ordinal()).unwrap(); //col.ordinal is of type usize. the query returns a number with sql type INT8, which corresponds with rust i64.
}

pub async fn buscarUsuario(nomUsuario: &String, dbPool: &sqlx::PgPool) -> sqlx::Result<Usuario> {
    let usuario: Result<Usuario, sqlx::Error>  = sqlx::query_as::<_, Usuario>("SELECT * FROM usuario WHERE nombreusuario = $1")
        .bind(nomUsuario)
        .fetch_one(dbPool)
        .await;

    return usuario;
}

pub async fn insertarUsuario(usuario: Usuario, dbPool: &sqlx::PgPool) -> sqlx::Result<PgQueryResult> {
    println!("{:?}", usuario.tipousuario);
    let res = sqlx::query("INSERT INTO usuario(email, nombrecompleto, nombreusuario, hashcontrasenna, idwallet, tipousuario) VALUES($1, $2, $3, $4, $5, $6)")
        .bind(usuario.email)
        .bind(usuario.nombrecompleto)
        .bind(usuario.nombreusuario)
        .bind(usuario.hashcontrasenna)
        .bind(usuario.idwallet)
        .bind(usuario.tipousuario as TipoUsuario)
        .execute(dbPool)
        .await;
    return res;
}