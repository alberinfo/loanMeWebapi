#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use crate::services::misc::{deserializeNaiveDateTime, serializeNaiveDateTime};

use super::{Prestamo::{LoanError, Prestamo}, usuario::{Usuario, TipoUsuario}};

#[derive(sqlx::Type, serde::Serialize, serde::Deserialize, Debug, Default)]
#[sqlx(type_name = "AcceptedBlockchains", rename_all = "lowercase")]
pub enum AcceptedBlockchains {
    #[default]
    Monero
}

#[derive(sqlx::FromRow, sqlx::Type, serde::Serialize, serde::Deserialize, Debug)]
pub struct PrestamoTxn {
    #[serde(skip_serializing, rename="LoanId")]
    pub fkPrestamo: i64,

    pub blockchain: AcceptedBlockchains,
    pub txnId: String,
}

impl PrestamoTxn {
    pub async fn getAllTxns(loanId: i64, dbPool: &sqlx::PgPool) -> Result<Vec<PrestamoTxn>, LoanError> {
        let data = sqlx::query_as::<_, PrestamoTxn>("SELECT * FROM PrestamoTxns WHERE \"fkPrestamo\" = $1")
            .bind(loanId)
            .fetch_all(dbPool)
            .await?;
        return Ok(data);
    }

    pub async fn addTxn(&self, user: &Usuario, dbPool: &sqlx::PgPool) -> Result<sqlx::postgres::PgQueryResult, LoanError> {
        if user.tipoUsuario.clone().unwrap() == TipoUsuario::Administrador {
            return Err(LoanError::InvalidUserType { found: Some(TipoUsuario::Administrador) })
        }
        
        let loan = Prestamo::getLoanById(self.fkPrestamo, dbPool).await?;

        match (user.tipoUsuario.clone().unwrap(), loan.fkPrestamista) {
            (TipoUsuario::Administrador, _) => return Err(LoanError::InvalidDate),
            (TipoUsuario::Prestamista, Some(x)) => return Err(LoanError::UserUnauthorized { expected: TipoUsuario::Prestamista, found: TipoUsuario::Prestamista }), //If the user is a loaner and the the loan already has an assigned prestamist
            (TipoUsuario::Prestatario, None /* If fkPrestamista is null then fkPrestatario is Some(x) */) => return Err(LoanError::UserUnauthorized { expected: TipoUsuario::Prestatario, found: TipoUsuario::Prestatario }),

            (_, _) => {} //anything not covered
        }
        
        let res = sqlx::query("INSERT INTO PrestamoTxns(\"fkPrestamo\", blockchain, \"txnId\") VALUES($1, $2, $3)")
            .bind(self.fkPrestamo)
            .bind(&self.blockchain as &AcceptedBlockchains)
            .bind(self.txnId.clone())
            .execute(dbPool)
            .await?;
        return Ok(res);
    }
}