#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use crate::services::misc::{deserializeNaiveDateTime, serializeNaiveDateTime};

#[derive(sqlx::Type, serde::Serialize, serde::Deserialize, Debug, Default)]
#[sqlx(type_name = "AcceptedBlockchains", rename_all = "lowercase")]
pub enum AcceptedBlockchains {
    #[default]
    Monero
}

#[derive(sqlx::FromRow, sqlx::Type, serde::Serialize, serde::Deserialize, Debug)]
pub struct PrestamoTxn {
    #[serde(skip)]
    pub fkPrestamo: i64,

    pub blockchain: AcceptedBlockchains,
    pub txnId: String,

    #[serde(deserialize_with = "deserializeNaiveDateTime", serialize_with = "serializeNaiveDateTime")]
    pub creationDate: chrono::NaiveDateTime
}