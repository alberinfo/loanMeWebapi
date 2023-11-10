#![allow(non_snake_case, non_camel_case_types)]
#![allow(clippy::needless_return)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use lettre::{AsyncSmtpTransport, transport::{smtp::{authentication::{Credentials, Mechanism}, PoolConfig}, self}, Tokio1Executor};

#[derive(Clone)]
pub struct mailingState {
    pub mailingPool: Option<AsyncSmtpTransport<Tokio1Executor>>
}

impl Default for mailingState {
    fn default() -> Self {
        return mailingState { mailingPool: None };
    }
}

impl mailingState {
    pub fn connect(&mut self) -> Result<(), transport::smtp::Error> { //What is the error type?
        let smtpUsername = &std::env::var("SMTP_USER").expect("No SMTP User defined in .env");
        let smtpPwd = &std::env::var("SMTP_PWD").expect("No SMTP PWD defined in .env");

        self.mailingPool = Some(
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&std::env::var("SMTP_URL").unwrap())?
            .port(587)
            .credentials(Credentials::new(
                smtpUsername.clone(),
                smtpPwd.clone()
            ))
            .authentication(vec![Mechanism::Plain])
            .pool_config(PoolConfig::new().max_size(10))
            .build()
        );

        return Ok(());
    }

    pub fn getConnection(&self) -> Option<&AsyncSmtpTransport<Tokio1Executor>> {
        return self.mailingPool.as_ref();
    } 
}