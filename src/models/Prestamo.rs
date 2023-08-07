#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use bigdecimal::BigDecimal;

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
    pub async fn getAllLoanOffers(dbPool: &sqlx::PgPool) -> sqlx::Result<Vec<Prestamo>> {
        let prestamos: Vec<Prestamo> = sqlx::query_as::<_, Prestamo>("SELECT * FROM PRESTAMO WHERE fkPrestatario IS NULL").fetch_all(dbPool).await?;
        return Ok(prestamos);
    }
}