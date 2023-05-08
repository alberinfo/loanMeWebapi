//use sqlx::{mssql::MssqlQueryResult};
//use sqlx::postgres;
use serde::{Serialize, Deserialize};

pub async fn getDbConnection() -> sqlx::PgPool {
    return sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap()).await.unwrap();
}