#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use bigdecimal::BigDecimal;
use super::usuario;

#[derive(thiserror::Error, Debug)]
pub enum LoanError {
    #[error("There was an error while executing a database query")]
    DbError(#[from] sqlx::Error),

    #[error("User provided has an invalid type {found:?}")]
    InvalidUserType {
        found: Option<usuario::TipoUsuario>
    },
    #[error("User is unauthorized for this operation (expected: ?, found: ?)")]
    UserUnauthorized {
        expected: usuario::TipoUsuario,
        found: usuario::TipoUsuario
    }
}

#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Prestamo {
    #[serde(skip_serializing, skip_deserializing)]
    pub id: i64,
    pub monto: BigDecimal,
    pub fechaCreacion: chrono::NaiveDate,
    pub interes: f64,
    pub plazoPago: chrono::NaiveDate,
    pub intervaloPago: String, //Likely to change
    pub riesgo: i32,

    #[serde(skip_serializing)]
    pub fkPrestatario: Option<i32>,
    pub fkPrestamista: Option<i32>
}

impl Prestamo {
    pub async fn getAllLoanOffers(dbPool: &sqlx::PgPool) -> Result<Vec<Prestamo>, LoanError> {
        let data = sqlx::query_as::<_, Prestamo>("SELECT * FROM Prestamo WHERE fkPrestatario IS NULL").fetch_all(dbPool).await?;
        return Ok(data);
    }

    pub async fn getAllLoanRequests(dbPool: &sqlx::PgPool) -> Result<Vec<Prestamo>, LoanError> {
        let data = sqlx::query_as::<_, Prestamo>("SELECT * FROM Prestamo WHERE fkPrestamista IS NULL").fetch_all(dbPool).await?;
        return Ok(data);
    }

    pub async fn getLoanById(&self, dbPool: &sqlx::PgPool) -> Result<Prestamo, LoanError> {
        let data = sqlx::query_as::<_, Prestamo>("SELECT * FROM Prestamo WHERE ID = $1")
            .bind(self.id)
            .fetch_one(dbPool)
            .await?;
        return Ok(data);
    }

    pub async fn createLoanOffer(&self, offerer: usuario::Usuario, dbPool: &sqlx::PgPool) -> Result<(), LoanError> {
        match offerer.tipousuario {
            None => return Err(LoanError::InvalidUserType { found: None }),
            Some(x) => match x {
                usuario::TipoUsuario::Administrador => return Err(LoanError::InvalidUserType { found: Some(usuario::TipoUsuario::Administrador) }),
                usuario::TipoUsuario::Prestatario => return Err(LoanError::UserUnauthorized { expected: usuario::TipoUsuario::Prestamista, found: usuario::TipoUsuario::Prestatario }),
                usuario::TipoUsuario::Prestamista => ()
            }
        }

        let result = sqlx::query("INSERT INTO Prestamo(monto, fechaCreacion, interes, plazoPago, intervaloPago, riesgo, fkPrestamista) VALUES($1, $2, $3, $4, $5, $6, $7)")
            .bind(&self.monto)
            .bind(self.fechaCreacion)
            .bind(self.interes)
            .bind(self.plazoPago)
            .bind(&self.intervaloPago)
            .bind(self.riesgo)
            .bind(offerer.id)
            .execute(dbPool)
            .await?;
        return Ok(());
    }

    pub async fn createLoanRequest(&self, requester: usuario::Usuario, dbPool: &sqlx::PgPool) -> Result<(), LoanError> {
        match requester.tipousuario {
            None => return Err(LoanError::InvalidUserType { found: None }),
            Some(x) => match x {
                usuario::TipoUsuario::Administrador => return Err(LoanError::InvalidUserType { found: Some(usuario::TipoUsuario::Administrador) }),
                usuario::TipoUsuario::Prestamista => return Err(LoanError::UserUnauthorized { expected: usuario::TipoUsuario::Prestatario, found: usuario::TipoUsuario::Prestamista }),
                usuario::TipoUsuario::Prestatario => ()
            }
        }

        let result = sqlx::query("INSERT INTO Prestamo(monto, fechaCreacion, interes, plazoPago, intervaloPago, riesgo, fkPrestatario) VALUES($1, $2, $3, $4, $5, $6, $7)")
            .bind(&self.monto)
            .bind(self.fechaCreacion)
            .bind(self.interes)
            .bind(self.plazoPago)
            .bind(&self.intervaloPago)
            .bind(self.riesgo)
            .bind(requester.id)
            .execute(dbPool)
            .await?;
        return Ok(());
    }
}