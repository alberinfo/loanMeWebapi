#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use super::usuario::Usuario;

pub enum MailType {
    SignupConfirm(Usuario, String), //User, ConfirmationId
    PwdRestore(String), //RestoreId
    Test,
    Reserved
}

impl MailType {
    
}