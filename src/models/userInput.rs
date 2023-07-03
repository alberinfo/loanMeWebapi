use super::usuario::Usuario;
use super::PerfilCrediticio::PerfilCrediticio;

#[derive(serde::Deserialize, Debug)]
pub struct userInput {
    pub Usuario: Usuario,
    pub perfil: Option<PerfilCrediticio>
}