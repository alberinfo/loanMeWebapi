#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use redis::AsyncCommands;
use crate::services::redisServer::DEFAULT_SESSION_EXPIRATION;
use crate::services::misc::generateRnd;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Session {
    #[serde(skip_serializing)]
    pub username: String, //which user does this session belong to?
    #[serde(rename="sessionId")]
    pub id: String,
    pub creationDate: Option<chrono::NaiveDateTime>
}

impl Session {
    pub async fn new(user: String) -> Session { //Create a new session for a given user
        let sessionIdHash = generateRnd(64).await.unwrap();

        let newSession = Session {
            username: user,
            id: sessionIdHash,
            creationDate: Some(chrono::Utc::now().naive_utc())
        };

        return newSession;
    }

    //TTL = Time to live
    pub async fn getTTL(sessionId: &String, redisConn: &mut redis::aio::ConnectionManager) -> redis::RedisResult<i64> {
        return redisConn.ttl(format!("{}{}", "sessionId", sessionId)).await;
    }

    pub async fn createSession(&self, redisConn: &mut redis::aio::ConnectionManager) -> redis::RedisResult<()> {
        //Generic params are: param1, param2, return type.
        redisConn.set_ex::<String, String, String>(format!("{}{}", "sessionId", self.id), self.username.clone(), DEFAULT_SESSION_EXPIRATION).await?;
        redisConn.set_ex::<String, String, String>(format!("{}{}", "sessionUser", self.username), self.id.clone(), DEFAULT_SESSION_EXPIRATION).await?;
        return Ok(());
    }

    //Gets the username given the session id
    pub async fn getSessionUserById(sessionId: &String, redisConn: &mut redis::aio::ConnectionManager) -> redis::RedisResult<String> {
        return redisConn.get::<String, String>(format!("{}{}", "sessionId", sessionId)).await;
    }

    pub async fn getSessionIdByUsername(username: &String, redisConn: &mut redis::aio::ConnectionManager) -> redis::RedisResult<String> {
        return redisConn.get::<String, String>(format!("{}{}", "sessionUser", username)).await;
    }

    //Check if user session exists
    pub async fn verifySessionById(sessionId: &String, redisConn: &mut redis::aio::ConnectionManager) -> bool {
        return redisConn.exists(format!("{}{}", "sessionId", sessionId)).await.unwrap();
    }

    pub async fn verifySessionByUsername(username: &String, redisConn: &mut redis::aio::ConnectionManager) -> bool {
        return redisConn.exists(format!("{}{}", "sessionUser", username)).await.unwrap();
    }

    pub async fn refreshSession(&self, redisConn: &mut redis::aio::ConnectionManager) -> redis::RedisResult<()> {
        let username = Session::getSessionUserById(&self.id, redisConn).await?;

        redisConn.expire(format!("{}{}", "sessionId", self.id), DEFAULT_SESSION_EXPIRATION).await?;
        redisConn.expire(format!("{}{}", "sessionUser", username), DEFAULT_SESSION_EXPIRATION).await?;
        return Ok(());
    }

    //When user logs out, probably.
    pub async fn deleteSession(sessionId: &String, redisConn: &mut redis::aio::ConnectionManager) -> redis::RedisResult<()> {
        let username = Session::getSessionUserById(sessionId, redisConn).await?;

        redisConn.del(format!("{}{}", "sessionId", sessionId)).await?;
        redisConn.del(format!("{}{}", "sessionUser", username)).await?;

        return Ok(());
    }
}