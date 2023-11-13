use std::collections::HashMap;

use super::PrestamoTxn::PrestamoTxn;
use super::usuario::Usuario;
use super::PerfilCrediticio::PerfilCrediticio;
use super::Prestamo::Prestamo;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct InputRequestRestorePwd {
    pub username: String
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct InputRestorePwd {
    pub newPwd: String
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct InputPerfilCrediticio {
    pub Usuario: Usuario,
    pub perfil: Option<PerfilCrediticio>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>
}

#[derive(serde::Deserialize, Debug)]
pub struct InputPrestamo {
    pub Loan: Prestamo,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>
}

#[derive(serde::Deserialize, Debug)]
pub struct InputProposal {
    pub LoanId: i64,
    pub walletId: Option<String>
}

#[derive(serde::Deserialize, Debug)]
pub struct InputTxn {
    pub txn: PrestamoTxn
}