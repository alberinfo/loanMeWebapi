#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use bigdecimal::BigDecimal;
use super::usuario;
use crate::services::misc::deserializeNaiveDateTime;

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

#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct Prestamo {
    #[serde(skip_deserializing)]
    pub id: i64,
    pub monto: BigDecimal,

    #[serde(skip_deserializing)]
    pub fechaCreacion: chrono::NaiveDateTime,
    
    pub interes: f64,

    #[serde(deserialize_with = "deserializeNaiveDateTime")]
    pub plazoPago: chrono::NaiveDateTime,
    pub intervaloPago: String, //Likely to change
    pub riesgo: i32,

    #[serde(skip_serializing)]
    pub fkPrestatario: Option<i64>,

    #[serde(skip_serializing)]
    pub fkPrestamista: Option<i64>
}

impl Prestamo {
    pub async fn getAllLoanOffers(dbPool: &sqlx::PgPool) -> Result<Vec<Prestamo>, LoanError> {
        let data = sqlx::query_as::<_, Prestamo>("SELECT * FROM Prestamo WHERE \"fkPrestatario\" IS NULL").fetch_all(dbPool).await?;
        return Ok(data);
    }

    pub async fn getAllLoanRequests(dbPool: &sqlx::PgPool) -> Result<Vec<Prestamo>, LoanError> {
        let data = sqlx::query_as::<_, Prestamo>("SELECT * FROM Prestamo WHERE \"fkPrestamista\" IS NULL").fetch_all(dbPool).await?;
        return Ok(data);
    }

    pub async fn getLoanById(id: i64, dbPool: &sqlx::PgPool) -> Result<Prestamo, LoanError> {
        let data = sqlx::query_as::<_, Prestamo>("SELECT * FROM Prestamo WHERE ID = $1")
            .bind(id)
            .fetch_one(dbPool)
            .await?;
        return Ok(data);
    }

    pub async fn createLoanOffer(&mut self, offerer: usuario::Usuario, dbPool: &sqlx::PgPool) -> Result<(), LoanError> {
        match offerer.tipoUsuario {
            None => return Err(LoanError::InvalidUserType { found: None }),
            Some(x) => match x {
                usuario::TipoUsuario::Administrador => return Err(LoanError::InvalidUserType { found: Some(usuario::TipoUsuario::Administrador) }),
                usuario::TipoUsuario::Prestatario => return Err(LoanError::UserUnauthorized { expected: usuario::TipoUsuario::Prestamista, found: usuario::TipoUsuario::Prestatario }),
                usuario::TipoUsuario::Prestamista => ()
            }
        }

        self.fechaCreacion = chrono::Utc::now().naive_utc();

        let _ = sqlx::query("INSERT INTO Prestamo(monto, \"fechaCreacion\", interes, \"plazoPago\", \"intervaloPago\", riesgo, \"fkPrestamista\") VALUES($1, $2, $3, $4, $5, $6, $7)")
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

    pub async fn createLoanRequest(&mut self, requester: usuario::Usuario, dbPool: &sqlx::PgPool) -> Result<(), LoanError> {
        match requester.tipoUsuario {
            None => return Err(LoanError::InvalidUserType { found: None }),
            Some(x) => match x {
                usuario::TipoUsuario::Administrador => return Err(LoanError::InvalidUserType { found: Some(usuario::TipoUsuario::Administrador) }),
                usuario::TipoUsuario::Prestamista => return Err(LoanError::UserUnauthorized { expected: usuario::TipoUsuario::Prestatario, found: usuario::TipoUsuario::Prestamista }),
                usuario::TipoUsuario::Prestatario => ()
            }
        }

        self.fechaCreacion = chrono::Utc::now().naive_utc();

        let _ = sqlx::query("INSERT INTO Prestamo(monto, \"fechaCreacion\", interes, \"plazoPago\", \"intervaloPago\", riesgo, \"fkPrestatario\") VALUES($1, $2, $3, $4, $5, $6, $7)")
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