//This file has methods for accessing redis server
#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use redis::AsyncCommands;
use crate::models::session;

pub static DEFAULT_SESSION_EXPIRATION: usize = 600; //600 secs --> 10 minutes

pub async fn getRedisConnection() -> redis::RedisResult<redis::aio::ConnectionManager> {
    let client: redis::Client = redis::Client::open(std::env::var("REDIS_URL").unwrap()).unwrap();
    let connManager: redis::RedisResult<redis::aio::ConnectionManager> = client.get_tokio_connection_manager().await;

    return connManager;
}

//TTL = Time to live
pub async fn getUserSessionTTL(sessionId: &String, redisConn: &mut redis::aio::ConnectionManager) -> redis::RedisResult<i64> {
    return redisConn.ttl(format!("{}{}", "sessionId", sessionId)).await;
}

pub async fn insertUserSession(sess: &session::Session, redisConn: &mut redis::aio::ConnectionManager) -> redis::RedisResult<String> {
    //Generic params are: param1, param2, return type.
    return redisConn.set_ex::<String, String, String>(format!("{}{}", "sessionId", sess.id), serde_json::to_string(sess).unwrap(), DEFAULT_SESSION_EXPIRATION).await;
}

//Check if user session exists
pub async fn verifyUserSession(sessionId: &String, redisConn: &mut redis::aio::ConnectionManager) -> bool {
    return redisConn.exists(format!("{}{}", "sessionId", sessionId)).await.unwrap();
}

pub async fn refreshUserSession(sessionId: &String, redisConn: &mut redis::aio::ConnectionManager) -> redis::RedisResult<()> {
    return redisConn.expire(format!("{}{}", "sessionId", sessionId), DEFAULT_SESSION_EXPIRATION).await;
}

//When user logs out, probably.
pub async fn deleteUserSession(sessionId: &String, redisConn: &mut redis::aio::ConnectionManager) -> redis::RedisResult<()> {
    return redisConn.del(format!("{}{}", "sessionId", sessionId)).await;
}