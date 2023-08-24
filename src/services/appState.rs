#![allow(non_snake_case, non_camel_case_types)]
#![allow(clippy::needless_return)]

use super::{db, redisServer, mailing};

#[derive(Clone)]
pub struct AppState {
    pub dbState: db::dbState,
    pub redisState: redisServer::redisState,
    pub mailingState: mailing::mailingState
}