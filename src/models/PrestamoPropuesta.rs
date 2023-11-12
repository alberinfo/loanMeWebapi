use super::{Prestamo::{Prestamo, LoanError}, usuario::{self, Usuario}};

#[derive(sqlx::FromRow, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename = "LoanProposal")]
pub struct PrestamoPropuesta {
    #[serde(rename = "LoanId")]
    pub fkPrestamo: i64,

    pub walletId: String,

    #[serde(rename = "UserId")]
    pub fkUsuario: i64
}

#[derive(serde::Serialize, Debug)]
#[serde(rename = "LoanProposalItem")]
pub struct PropuestaItem {
    #[serde(rename = "LoanId")]
    pub fkPrestamo: i64,
    
    #[serde(rename = "User")]
    pub usuario: Usuario
}

impl PrestamoPropuesta {
    pub async fn proposeCompleteLoan(LoanId: i64, walletId: Option<String>, user: &usuario::Usuario, dbPool: &sqlx::PgPool) -> Result<(), LoanError> {
        let loan = Prestamo::getLoanById(LoanId, dbPool).await?;

        if Some(user.id) == loan.fkPrestamista || Some(user.id) == loan.fkPrestatario {
            return Err(LoanError::InvalidUser);
        }

        let _ = match &user.tipoUsuario {
            None => return Err(LoanError::InvalidUserType { found: None }),
            Some(x) => match (x, loan.fkPrestamista, loan.fkPrestatario) {
                (usuario::TipoUsuario::Administrador, _, _) => return Err(LoanError::InvalidUserType { found: Some(usuario::TipoUsuario::Administrador) }),
                (usuario::TipoUsuario::Prestamista, Some(_), _) => return Err(LoanError::UserUnauthorized { expected: None, found: Some(usuario::TipoUsuario::Prestamista) }),
                (usuario::TipoUsuario::Prestatario, _, Some(_)) => return Err(LoanError::UserUnauthorized { expected: None, found: Some(usuario::TipoUsuario::Prestatario) }),
                (usuario::TipoUsuario::Prestamista, None, _) => sqlx::query("INSERT INTO PrestamoPropuesta(\"fkPrestamo\", \"fkUsuario\") VALUES($1, $2)").bind(LoanId).bind(user.id).execute(dbPool).await?,
                (usuario::TipoUsuario::Prestatario, _, None) => sqlx::query("INSERT INTO PrestamoPropuesta(\"fkPrestamo\", \"walletId\", \"fkUsuario\") VALUES($1, $2, $3)").bind(LoanId).bind(walletId).bind(user.id).execute(dbPool).await?,
            }
        };

        return Ok(());
    }

    pub async fn getLoanProposalById(LoanId: i64, UserId: i64, dbPool: &sqlx::PgPool) -> Result<Option<PrestamoPropuesta>, LoanError> {
        let data = sqlx::query_as::<_, PrestamoPropuesta>("SELECT * FROM PrestamoPropuesta WHERE \"fkPrestamo\" = $1 AND \"fkUsuario\" = $2")
            .bind(LoanId)
            .bind(UserId)
            .fetch_optional(dbPool)
            .await?;

        return Ok(data);
    }

    pub async fn getAllLoanProposalsForUser(UserId: i64, dbPool: &sqlx::PgPool) -> Result<Vec<PrestamoPropuesta>, LoanError> {
        let data = sqlx::query_as::<_, PrestamoPropuesta>("SELECT * FROM PrestamoPropuesta WHERE \"fkPrestamo\" IN (SELECT ID FROM Prestamo WHERE \"fkPrestamista\" = $1 OR \"fkPrestatario\" = $1) ")
            .bind(UserId)
            .fetch_all(dbPool)
            .await?;

        return Ok(data);
    }

    pub async fn clearLoanProposals(LoanId: i64, dbPool: &sqlx::PgPool) -> Result<(), LoanError> {
        let _ = sqlx::query("DELETE FROM PrestamoPropuesta WHERE \"fkPrestamo\" = $1 ").bind(LoanId).execute(dbPool).await?;
        Ok(())
    }
}