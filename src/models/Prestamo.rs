#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

#[derive(sqlx::FromRow, sqlx::Type, serde::Serialize, serde::Deserialize, serde::Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Prestamo {
    #[serde(skip_serializing, skip_deserializing)]
    id: i64,
    monto: f32,
    fechaCreacion: chrono::NaiveDate,
    interes: f64,
    plazoPago: chrono::NaiveDate,
    intervaloPago: String, //Likely to change
    riesgo: i32,
    fkPrestatario: Option<i32>,
    fkPrestamista: Option<i32>
}