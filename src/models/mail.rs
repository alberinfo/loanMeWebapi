#![allow(non_snake_case)]
#![allow(clippy::needless_return)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use lettre::{Message, message::header::ContentType, AsyncSmtpTransport, Tokio1Executor, AsyncTransport, transport::smtp::response::Response};
use redis::AsyncCommands;

use super::usuario::Usuario;
use crate::services::misc::generateRnd;
use crate::services::redisServer::{DEFAULT_MAILCONF_EXPIRATION, DEFAULT_PWDRESTORE_EXPIRATION};


#[derive(thiserror::Error, Debug)]
pub enum MailError {
    #[error("Email service test.")]
    Test,

    #[error("There was an error while parsing the email address")]
    AddressError(#[from] lettre::address::AddressError),

    #[error("There was an error while parsing the email body")]
    EmailError(#[from] lettre::error::Error),

    #[error("There was an error while sending the email")]
    SendError(#[from] lettre::transport::smtp::Error)
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum Mail {
    SignupConfirm(Usuario, String), //User, CreditHistory ConfirmationId
    PwdRestore(Usuario, String), //User, RestoreId
    LoanProposal(Usuario, Usuario, i64), //User that created the loan, loan proposer, loan id.
    LoanProposalAccepted(Usuario, i64), //User that proposed completion, loan id
    Test
}

impl Mail {
    pub async fn get(IdType: &str, id: &String, redisConn: &mut redis::aio::ConnectionManager) -> Result<Mail, redis::RedisError> {
        match IdType {
            "confirmationId" => {
                let userJson = redisConn.get_del::<String, String>(format!("{}{}", "confirmationId", id)).await?;
                let mut user: Usuario = serde_json::from_str(&userJson).unwrap();
                user.contrasenna.pop(); //remover el "*"
                return Ok(Mail::SignupConfirm(user, id.to_string()));
            },
            "restoreId" => {
                let userJson = redisConn.get_del::<String, String>(format!("{}{}", "restoreId", id)).await?;
                let mut user: Usuario = serde_json::from_str(&userJson).unwrap();
                user.contrasenna.pop(); //remover el "*"
                return Ok(Mail::PwdRestore(user, id.to_string()));
            },
            _ => {
                return Ok(Mail::Test);
            }
        }
    }

    //create the id and store it in redis
    pub async fn save(&mut self, redisConn: &mut redis::aio::ConnectionManager) -> redis::RedisResult<()> {
        match self {
            Mail::SignupConfirm(Usuario, ConfirmationId) => {
                std::mem::swap(ConfirmationId, &mut generateRnd(64).await.unwrap()); //ConfirmationId = &mut generateRnd(64).await.unwrap();
                Usuario.contrasenna.push('*');
                redisConn.set_ex::<String, String, String>(format!("{}{}", "confirmationId", ConfirmationId), serde_json::to_string(Usuario).unwrap(), DEFAULT_MAILCONF_EXPIRATION).await?;
            },
            Mail::PwdRestore(Usuario, RestoreId) => {
                std::mem::swap(RestoreId, &mut generateRnd(64).await.unwrap()); //RestoreId = &mut generateRnd(64).await.unwrap();
                Usuario.contrasenna.push('*');
                redisConn.set_ex::<String, String, String>(format!("{}{}", "restoreId", RestoreId), serde_json::to_string(Usuario).unwrap(), DEFAULT_PWDRESTORE_EXPIRATION).await?;
            },
            Mail::LoanProposal(_, _, _) | Mail::LoanProposalAccepted(_, _) | Mail::Test => {}
        }

        return Ok(());
    }

    pub async fn send(&self, mailingPool: &AsyncSmtpTransport<Tokio1Executor>) -> Result<Response, MailError> {
        let msg: Message = match self {
            Mail::SignupConfirm(Usuario, ConfirmationId) => {
                Message::builder()
                    .from("loanMe <no-reply@loanMe.com>".parse()?)
                    .to(format!("{} <{}>", Usuario.nombreCompleto, Usuario.email).parse()?)
                    .subject("Confirm your signup in order to use your account")
                    .header(ContentType::TEXT_PLAIN)
                    .body(format!("http://localhost:4433/validateUser/?token={ConfirmationId}"))?
            },
            Mail::PwdRestore(Usuario, RestoreId) => {
                Message::builder()
                    .from("loanMe <no-reply@loanMe.com>".parse()?)
                    .to(format!("{} <{}>", Usuario.nombreCompleto, Usuario.email).parse()?)
                    .subject("Restore pwd")
                    .header(ContentType::TEXT_PLAIN)
                    .body(format!("http://localhost:4433/restorePwd/?token={RestoreId}"))?
            },
            Mail::LoanProposal(LoanCreator, LoanCompleter, LoanId) => {
                Message::builder()
                    .from("loanMe <no-reply@loanMe.com>".parse()?)
                    .to(format!("{} <{}>", LoanCreator.nombreCompleto, LoanCreator.email).parse()?)
                    .subject("Loan completion proposal")
                    .header(ContentType::TEXT_PLAIN)
                    .body(format!("User {} wants to be your other half! See your loan here http://localhost:4433/getLoanById/{LoanId}", LoanCompleter.nombreUsuario))?
            },
            Mail::LoanProposalAccepted(LoanCompleter, LoanId) => {
                Message::builder()
                    .from("loanMe <no-reply@loanMe.com>".parse()?)
                    .to(format!("{} <{}>", LoanCompleter.nombreCompleto, LoanCompleter.email).parse()?)
                    .subject("Loan proposal accepted")
                    .header(ContentType::TEXT_PLAIN)
                    .body(format!("Your loan completion proposal has been accepted! See the loan here http://localhost:4433/getLoanById/{LoanId}"))?
            }
            Mail::Test => {
                return Err(MailError::Test);
            }
        };

        return Ok(mailingPool.send(msg).await?);
    }
}