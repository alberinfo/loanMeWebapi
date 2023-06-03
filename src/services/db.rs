#![allow(non_snake_case, non_camel_case_types)]
#![allow(clippy::needless_return)]

use sqlx::{postgres::PgRow, Row, Column};

#[derive(Clone)]
pub struct dbState {
    pub dbPool: Option<sqlx::PgPool>
}

impl dbState {
    pub fn new() -> dbState {
        let newState = dbState {
            dbPool: None
        };
        return newState;
    }

    pub async fn connect(&mut self) -> sqlx::Result<()> {
        self.dbPool = Some(sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap()).await?);
        return Ok(());
    }

    pub fn getConnection(&self) -> Option<&sqlx::PgPool> {
        if self.dbPool.is_none() {
            return None;
        }
        return Some(self.dbPool.as_ref().unwrap());
    }

    pub async fn getTableCount(&self) -> i64 {
        let row: PgRow = sqlx::query("SELECT COUNT(*) from information_schema.tables where table_schema = 'public'").fetch_one(self.dbPool.as_ref().unwrap()).await.unwrap();
        let col = row.column(0); //We know there is only one column for this query
        return row.try_get::<i64, usize>(col.ordinal()).unwrap(); //col.ordinal is of type usize. the query returns a number with sql type INT8, which corresponds with rust i64.
    }

    pub async fn migrateDb(&self) -> sqlx::Result<()> {
        let tableCount = self.getTableCount().await;
        if tableCount == 0 || tableCount == 1 { //assuming tablecount = 0 means db was just created, and = 1 means just _sqlx_migrations exists
            let res = sqlx::migrate!("./migrations").run(self.dbPool.as_ref().unwrap()).await?;
        }
        return Ok(());
    }
}