use std::collections::HashMap;

use super::usuario::Usuario;
use super::PerfilCrediticio::PerfilCrediticio;

#[derive(serde::Deserialize, Debug)]
pub struct UserInput {
    pub Usuario: Usuario,
    pub perfil: Option<PerfilCrediticio>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>
}