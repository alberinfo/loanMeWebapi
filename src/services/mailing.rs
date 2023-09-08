#![allow(non_snake_case, non_camel_case_types)]
#![allow(clippy::needless_return)]

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
    pub async fn connect(&mut self) -> Result<(), transport::smtp::Error> { //What is the error type?
        let smtpUsername = &std::env::var("SMTP_USER").unwrap();
        let smtpPwd = &std::env::var("SMTP_PWD").unwrap();

        self.mailingPool = Some(
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&std::env::var("SMTP_URL").unwrap())?
            .port(587)
            .credentials(Credentials::new(
                smtpUsername.to_owned(),
                smtpPwd.to_owned()
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