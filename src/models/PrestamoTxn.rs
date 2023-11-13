#![allow(non_snake_case)]
#![allow(clippy::needless_return)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use strum::EnumIter;

use super::{Prestamo::{LoanError, Prestamo}, usuario::{Usuario, TipoUsuario}};

#[derive(sqlx::Type, serde::Serialize, serde::Deserialize, Debug, EnumIter, Default, Clone)]
#[sqlx(type_name = "AcceptedBlockchains")]
pub enum AcceptedBlockchains {
    #[default]
    Monero,
    Bitcoin,
    BitcoinTestnet,
    Ethereum,
    EthereumTestnet
}

#[derive(sqlx::FromRow, sqlx::Type, serde::Serialize, serde::Deserialize, Debug)]
pub struct PrestamoTxn {
    #[serde(skip_serializing, rename="LoanId")]
    pub fkPrestamo: i64,
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
        if user.tipoUsuario == Some(TipoUsuario::Administrador) {
            return Err(LoanError::InvalidUserType { found: Some(TipoUsuario::Administrador) })
        }
        
        let loan = Prestamo::getLoanById(self.fkPrestamo, dbPool).await?;

        match (&user.tipoUsuario, loan.fkPrestamista, loan.fkPrestatario) {
            (Some(TipoUsuario::Administrador), _, _) => return Err(LoanError::InvalidDate),
            (Some(TipoUsuario::Prestamista), _, None) => return Err(LoanError::UserUnauthorized { expected: None, found: Some(TipoUsuario::Prestamista) }), //If the user is a loaner and the the loan already has an assigned prestamist
            (Some(TipoUsuario::Prestatario), None, _) => return Err(LoanError::UserUnauthorized { expected: None, found: Some(TipoUsuario::Prestatario) }),

            (_, _, _) => {} //anything not covered
        }
        
        let res = sqlx::query("INSERT INTO PrestamoTxns(\"fkPrestamo\", \"txnId\") VALUES($1, $2)")
            .bind(self.fkPrestamo)
            .bind(&self.txnId)
            .execute(dbPool)
            .await?;
        return Ok(res);
    }
}