use std::collections::HashMap;

use super::usuario::Usuario;
use super::PerfilCrediticio::PerfilCrediticio;
use super::Prestamo::Prestamo;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct InputPerfilCrediticio {
    pub Usuario: Usuario,
    pub perfil: Option<PerfilCrediticio>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>
}

#[derive(serde::Deserialize, Debug)]
pub struct InputPrestamo {
    pub Usuario: Usuario,
    pub Loan: Prestamo,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>
}