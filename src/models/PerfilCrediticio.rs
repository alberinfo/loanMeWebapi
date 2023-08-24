#![allow(non_snake_case)]
#![allow(clippy::needless_return)]



#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct PerfilCrediticio {
    #[serde(skip_serializing, skip_deserializing)]
    pub id: i64,
    #[serde(skip_serializing, skip_deserializing)]
    pub fkusuario: i64,
    #[serde(skip_serializing)]
    pub dni: String,

    pub historialcrediticio: Option<String>,
    pub extractobancario: Option<String>,
    pub comprobantedeingreso: Option<String>,
    pub descripcionfinanciera: Option<String>,
}

impl PerfilCrediticio {
    pub async fn get(fkUsuario: i64, dbPool: &sqlx::PgPool) -> sqlx::Result<PerfilCrediticio> {
        let PerfilCrediticio: sqlx::Result<PerfilCrediticio> = sqlx::query_as::<_, PerfilCrediticio>("SELECT * FROM perfilcrediticio WHERE fkUsuario = $1")
            .bind(fkUsuario)
            .fetch_one(dbPool)
            .await;

        return PerfilCrediticio;
    }

    pub async fn save(&self, dbPool: &sqlx::PgPool) -> sqlx::Result<sqlx::postgres::PgQueryResult> {
        let res = sqlx::query("INSERT INTO perfilcrediticio(fkUsuario, dni, historialcrediticio, extractobancario, comprobantedeingreso, descripcionfinanciera) VALUES($1, $2, $3, $4, $5, $6)")
            .bind(&self.fkusuario)
            .bind(&self.dni)
            .bind(&self.historialcrediticio)
            .bind(&self.extractobancario)
            .bind(&self.comprobantedeingreso)
            .bind(&self.descripcionfinanciera)
            .execute(dbPool)
            .await;
        return res;
    }

    pub async fn update(&self, dbPool: &sqlx::PgPool) -> sqlx::Result<sqlx::postgres::PgQueryResult> {
        let res = sqlx::query("UPDATE perfilcrediticio SET historialcrediticio = $1, extractobancario = $2, comprobantedeingreso = $3, descripcionfinanciera = $4 WHERE id = $4")
            .bind(&self.historialcrediticio)
            .bind(&self.extractobancario)
            .bind(&self.comprobantedeingreso)
            .bind(&self.descripcionfinanciera)
            .bind(&self.id)
            .execute(dbPool)
            .await;
        return res;
    }
}