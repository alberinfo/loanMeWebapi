#![allow(non_snake_case)]
#![allow(clippy::needless_return)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct PerfilCrediticio {
    #[serde(skip)]
    pub id: i64,
    #[serde(skip)]
    pub fkUsuario: i64,

    pub dni: String,

    pub historialCrediticio: Option<String>,
    pub extractoBancario: Option<String>,
    pub comprobanteDeIngreso: Option<String>,
    pub descripcionFinanciera: Option<String>,
}

impl PerfilCrediticio {
    pub async fn get(fkUsuario: i64, dbPool: &sqlx::PgPool) -> sqlx::Result<PerfilCrediticio> {
        let PerfilCrediticio: sqlx::Result<PerfilCrediticio> = sqlx::query_as::<_, PerfilCrediticio>("SELECT * FROM perfilCrediticio WHERE \"fkUsuario\" = $1")
            .bind(fkUsuario)
            .fetch_one(dbPool)
            .await;

        return PerfilCrediticio;
    }

    pub async fn save(&self, dbPool: &sqlx::PgPool) -> sqlx::Result<sqlx::postgres::PgQueryResult> {
        let res = sqlx::query("INSERT INTO perfilCrediticio(\"fkUsuario\", dni, \"historialCrediticio\", \"extractoBancario\", \"comprobanteDeIngreso\", \"descripcionFinanciera\") VALUES($1, $2, $3, $4, $5, $6)")
            .bind(self.fkUsuario)
            .bind(&self.dni)
            .bind(&self.historialCrediticio)
            .bind(&self.extractoBancario)
            .bind(&self.comprobanteDeIngreso)
            .bind(&self.descripcionFinanciera)
            .execute(dbPool)
            .await;
        return res;
    }

    pub async fn update(&self, dbPool: &sqlx::PgPool) -> sqlx::Result<sqlx::postgres::PgQueryResult> {
        let res = sqlx::query("UPDATE perfilCrediticio SET \"historialCrediticio\" = $1, \"extractoBancario\" = $2, \"comprobanteDeIngreso\" = $3, \"descripcionFinanciera\" = $4 WHERE id = $4")
            .bind(&self.historialCrediticio)
            .bind(&self.extractoBancario)
            .bind(&self.comprobanteDeIngreso)
            .bind(&self.descripcionFinanciera)
            .bind(self.id)
            .execute(dbPool)
            .await;
        return res;
    }
}