#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use bigdecimal::BigDecimal;
use super::{usuario, PrestamoTxn::PrestamoTxn};
use crate::services::misc::{deserializeNaiveDateTime, serializeNaiveDateTime};

#[derive(thiserror::Error, Debug)]
pub enum LoanError {
    #[error("There was an error while executing a database query")]
    DbError(#[from] sqlx::Error),

    #[error("The provided date is invalid")]
    InvalidDate,

    #[error("User is not valid for this operation")]
    InvalidUser,

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
pub struct Prestamo {
    #[serde(skip_deserializing)]
    pub id: i64,
    pub monto: BigDecimal,

    #[serde(skip_deserializing, serialize_with = "serializeNaiveDateTime")]
    pub fechaCreacion: chrono::NaiveDateTime,
    
    pub interes: f64,

    #[serde(deserialize_with = "deserializeNaiveDateTime", serialize_with = "serializeNaiveDateTime")]
    pub plazoPago: chrono::NaiveDateTime,
    pub intervaloPago: String, //Likely to change
    pub riesgo: i32,

    #[serde(skip_serializing)]
    pub fkPrestatario: Option<i64>,

    #[serde(skip_serializing)]
    pub fkPrestamista: Option<i64>
}

#[derive(serde::Serialize, Debug)]
pub struct LoanItem {
    pub loan: Prestamo,
    pub txns: Vec<PrestamoTxn>,
    pub user: super::usuario::Usuario
}

#[derive(sqlx::FromRow, serde::Serialize, Default, Debug)]
pub struct PrestamoPropuesta {
    pub fkPrestamo: i64,
    pub fkUsuario: i64
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

    pub async fn getAllLoansFromUser(UserId: i64, dbPool: &sqlx::PgPool) -> Result<Vec<Prestamo>, LoanError> {
        let data = sqlx::query_as::<_, Prestamo>("SELECT * FROM Prestamo WHERE \"fkPrestamista\" = $1 OR \"fkPrestatario\" = $1 ")
            .bind(UserId)
            .fetch_all(dbPool)
            .await?;

        return Ok(data);
    }

    pub async fn getLoanById(id: i64, dbPool: &sqlx::PgPool) -> Result<Prestamo, LoanError> {
        let data = sqlx::query_as::<_, Prestamo>("SELECT Prestamo.*, (SELECT * FROM PrestamoTxn WHERE \"fkPrestamo\" = $1) AS txns FROM Prestamo WHERE ID = $1")
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

        if self.fechaCreacion.timestamp() >= self.plazoPago.timestamp() { //si el plazo de pago esta puesto como previo al momento de creacion del prestamo
            return Err(LoanError::InvalidDate);
        }

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

        if self.fechaCreacion.timestamp() >= self.plazoPago.timestamp() { //si el plazo de pago esta puesto como previo al momento de creacion del prestamo
            return Err(LoanError::InvalidDate);
        }

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

    pub async fn proposeCompleteLoan(LoanId: i64, user: &usuario::Usuario, dbPool: &sqlx::PgPool) -> Result<(), LoanError> {
        let loan = Prestamo::getLoanById(LoanId, dbPool).await?;

        let res = match &user.tipoUsuario {
            None => return Err(LoanError::InvalidUserType { found: None }),
            Some(x) => match (x, loan.fkPrestamista, loan.fkPrestatario) {
                (usuario::TipoUsuario::Administrador, _, _) => return Err(LoanError::InvalidUserType { found: Some(usuario::TipoUsuario::Administrador) }),
                (usuario::TipoUsuario::Prestamista, Some(_), _) => return Err(LoanError::UserUnauthorized { expected: usuario::TipoUsuario::Prestatario, found: usuario::TipoUsuario::Prestamista }),
                (usuario::TipoUsuario::Prestatario, _, Some(_)) => return Err(LoanError::UserUnauthorized { expected: usuario::TipoUsuario::Prestamista, found: usuario::TipoUsuario::Prestatario }),
                (usuario::TipoUsuario::Prestamista, None, _) | (usuario::TipoUsuario::Prestatario, _, None) => {
                    if loan.fkPrestatario.unwrap() == user.id {
                        return Err(LoanError::InvalidUser)
                    }
                    sqlx::query("INSERT PrestamoPropuesta(\"fkPrestamo\", \"fkUsuario\") VALUES($1, $2)").bind(LoanId).bind(user.id).execute(dbPool).await?
                },

            }
        };

        return Ok(());
    }

    //////// HACEEEEEEEEER
    /*pub async fn getAllLoanProposalsForUser(UserId: i64, dbPool: &sqlx::PgPool) -> Result<Vec<PrestamoPropuesta>, LoanError> {
        let data = sqlx::query_as::<_, PrestamoPropuesta>("SELECT * FROM PrestamoPropuesta WHERE \"fk\" = $1 OR \"fkPrestatario\" = $1 ")
            .bind(UserId)
            .fetch_all(dbPool)
            .await?;

        return Ok(data);
    }*/

    pub async fn completeLoan(LoanId: i64, user: &usuario::Usuario, dbPool: &sqlx::PgPool) -> Result<(), LoanError> {
        let loan = Prestamo::getLoanById(LoanId, dbPool).await?;

        let proposal = sqlx::query_as::<_, PrestamoPropuesta>("SELECT * FROM PrestamoPropuesta WHERE \"fkPrestamo\" = $1 AND \"fkUsuario\" = $2")
            .bind(LoanId)
            .bind(user.id)
            .fetch_optional(dbPool)
            .await?;

        if let None = proposal {
            return Err(LoanError::InvalidUser);
        }

        let proposal = proposal.unwrap();
        
        let _ = match (user.tipoUsuario.clone().unwrap(), loan.fkPrestamista, loan.fkPrestatario) {
            (usuario::TipoUsuario::Prestamista, None, Some(_)) => sqlx::query("UPDATE Prestamo SET \"fkPrestamista\" = $1 WHERE ID = $2").bind(user.id).bind(LoanId).execute(dbPool).await?,
            (usuario::TipoUsuario::Prestatario, Some(_), None) => sqlx::query("UPDATE Prestamo SET \"fkPrestatario\" = $1 WHERE ID = $2").bind(user.id).bind(LoanId).execute(dbPool).await?,
            (_, _, _) => return Err(LoanError::InvalidUser)
        };

        let _ = sqlx::query("DELETE FROM PrestamoPropuestas WHERE \"fkPrestamo\" = $1 ").bind(LoanId).execute(dbPool).await?;

        return Ok(());
    }
}