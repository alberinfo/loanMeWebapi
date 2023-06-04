#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use argon2::password_hash::rand_core::{RngCore, OsRng};
use sha2::{Sha512, Digest};
use redis::AsyncCommands;
use crate::services::redisServer::DEFAULT_SESSION_EXPIRATION;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Session {
    #[serde(skip_serializing)]
    pub username: String, //which user does this session belong to?
    #[serde(rename="sessionId")]
    pub id: String,
    pub creationDate: Option<chrono::DateTime<chrono::Utc>>
}

impl Session {
    pub async fn new(user: String) -> Session { //Create a new session for a given user
        let sessionIdHash = tokio::task::spawn_blocking(|| {
            let mut buf = [0u8; 32]; //generate 256 bits of entropy
            OsRng.fill_bytes(&mut buf);
    
            return format!("{:X}", Sha512::digest(buf));
        }).await.unwrap();

        let newSession = Session {
            username: user,
            id: sessionIdHash,
            creationDate: Some(chrono::Utc::now())
        };

        return newSession;
    }

    //TTL = Time to live
    pub async fn getTTL(&self, redisConn: &mut redis::aio::ConnectionManager) -> redis::RedisResult<i64> {
        return redisConn.ttl(format!("{}{}", "sessionId", self.id)).await;
    }

    pub async fn createSession(&self, redisConn: &mut redis::aio::ConnectionManager) -> redis::RedisResult<()> {
        //Generic params are: param1, param2, return type.
        redisConn.set_ex::<String, String, String>(format!("{}{}", "sessionId", self.id), self.username.clone(), DEFAULT_SESSION_EXPIRATION).await?;
        redisConn.set_ex::<String, String, String>(format!("{}{}", "sessionUser", self.username), self.id.clone(), DEFAULT_SESSION_EXPIRATION).await?;
        return Ok(());
    }

    //Gets the username given the session id
    pub async fn getSessionUserById(&self, redisConn: &mut redis::aio::ConnectionManager) -> redis::RedisResult<String> {
        return redisConn.get::<String, String>(format!("{}{}", "sessionId", self.id)).await;
    }

    pub async fn getSessionIdByUsername(&self, redisConn: &mut redis::aio::ConnectionManager) -> redis::RedisResult<String> {
        return redisConn.get::<String, String>(format!("{}{}", "sessionUser", self.username)).await;
    }

    //Check if user session exists
    pub async fn verifySessionById(&self, redisConn: &mut redis::aio::ConnectionManager) -> bool {
        return redisConn.exists(format!("{}{}", "sessionId", self.id)).await.unwrap();
    }

    pub async fn verifySessionByUsername(&self, redisConn: &mut redis::aio::ConnectionManager) -> bool {
        return redisConn.exists(format!("{}{}", "sessionUser", self.username)).await.unwrap();
    }

    pub async fn refreshSession(&self, redisConn: &mut redis::aio::ConnectionManager) -> redis::RedisResult<()> {
        let username = self.getSessionUserById(redisConn).await?;

        redisConn.expire(format!("{}{}", "sessionId", self.id), DEFAULT_SESSION_EXPIRATION).await?;
        redisConn.expire(format!("{}{}", "sessionUser", username), DEFAULT_SESSION_EXPIRATION).await?;
        return Ok(());
    }

    //When user logs out, probably.
    pub async fn deleteSession(&self, redisConn: &mut redis::aio::ConnectionManager) -> redis::RedisResult<()> {
        let username = self.getSessionUserById(redisConn).await?;

        redisConn.del(format!("{}{}", "sessionId", self.id)).await?;
        redisConn.del(format!("{}{}", "sessionUser", username)).await?;

        return Ok(());
    }
}