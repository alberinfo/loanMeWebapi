//This file has methods for accessing redis server
#![allow(non_snake_case)]

pub async fn getRedisConnection() -> redis::RedisResult<redis::aio::ConnectionManager> {
    let client: redis::Client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let connManager: redis::RedisResult<redis::aio::ConnectionManager> = client.get_tokio_connection_manager().await;//redis::aio::ConnectionManager::new(client).await;

    return connManager;
}

pub async fn insertUserSession() -> String {
    return "todo".to_string();
}