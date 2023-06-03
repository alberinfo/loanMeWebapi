//This file has methods for accessing redis server
#![allow(non_snake_case, non_camel_case_types)]
#![allow(clippy::needless_return)]

pub static DEFAULT_SESSION_EXPIRATION: usize = 600; //600 secs --> 10 minutes

#[derive(Clone)]
pub struct redisState {
    pub redisConn: Option<redis::aio::ConnectionManager>
}

impl Default for redisState {
    fn default() -> Self {
        return redisState { redisConn: None };
    }
}

impl redisState {    
    pub async fn connect(&mut self) -> redis::RedisResult<()> {
        let client: redis::Client = redis::Client::open(std::env::var("REDIS_URL").unwrap()).unwrap();
        self.redisConn = Some(client.get_tokio_connection_manager().await?);
    
        return Ok(());
    }
    
    pub fn getConnection(&mut self) -> Option<&mut redis::aio::ConnectionManager> {
        return self.redisConn.as_mut();
    }
}