//This file has methods for accessing redis server
#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

pub static DEFAULT_SESSION_EXPIRATION: usize = 600; //600 secs --> 10 minutes

#[derive(Clone)]
pub struct redisState {
    pub redisConn: Option<redis::aio::ConnectionManager>
}

impl redisState {    
    pub fn new() -> redisState {
        let newState = redisState {
            redisConn: None
        };
        return newState;
    }

    pub async fn connect(&mut self) -> redis::RedisResult<()> {
        let client: redis::Client = redis::Client::open(std::env::var("REDIS_URL").unwrap()).unwrap();
        self.redisConn = Some(client.get_tokio_connection_manager().await?);
    
        return Ok(());
    }
    
    pub fn getConnection(&mut self) -> Option<&mut redis::aio::ConnectionManager> {
        if self.redisConn.is_none() {
            return None;
        }

        return Some(self.redisConn.as_mut().unwrap());
    }
}