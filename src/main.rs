use mobc_pool::MobcPool;
use std::convert::Infallible;
use warp::{Filter, Rejection, Reply};
use thiserror::Error;

mod direct;
mod mobc_pool;

type WebResult<T> = std::result::Result<T, Rejection>;
type Result<T> = std::result::Result<T, Error>;

const REDIS_CON_STRING: &str = "redis://127.0.0.1";

#[tokio::main]
async fn main() {
    let redis_client = redis::Client::open(REDIS_CON_STRING)
        .expect("can create redis client");

    let mobc_pool = mobc_pool::connect()
        .await
        .expect("can create mobc poool");

    let direct_route = warp::path!("direct")
        .and(with_redis_client(redis_client.clone()))
        .and_then(direct_handler);

    let mobc_route = warp::path!("mobc")
        .and(with_mobc_pool(mobc_pool.clone()))
        .and_then(mobc_handler);

    let routes = direct_route
        .or(mobc_route);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;

}

fn with_redis_client(
    client: redis::Client
) -> impl Filter<Extract = (redis::Client,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

fn with_mobc_pool(
    pool: MobcPool,
) -> impl Filter<Extract = (MobcPool,), Error = Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

async fn direct_handler(client: redis::Client) -> WebResult<impl Reply> {
    let mut con = direct::get_con(client)
        .await
        .map_err(|e| warp::reject::custom(e))?;

    direct::set_str(&mut con, "hello", "direct_world", 120)
        .await
        .map_err(|e| warp::reject::custom(e))?;

    let value = direct::get_str(&mut con, "hello")
        .await
        .map_err(|e| warp::reject::custom(e))?;

    Ok(value)
}

async fn mobc_handler(pool: MobcPool) -> WebResult<impl Reply> {
    mobc_pool::set_str(&pool, "mobc_hello", "mobc_world", 120)
        .await
        .map_err(|e| warp::reject::custom(e))?;

    let value = mobc_pool::get_str(&pool, "mobc_hello")
        .await
        .map_err(|e| warp::reject::custom(e))?;

    Ok(value)
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("direct redis error: {0}")]
    DirectError(#[from] DirectError),
    #[error("mobc error: {0}")]
    MobcError(#[from] MobcError),
}

#[derive(Error, Debug)]
pub enum DirectError {
    #[error("error parsing string from redis result: {0}")]
    RedisTypeError(redis::RedisError),
    #[error("error creating Redis client: {0}")]
    RedisClientError(redis::RedisError),
    #[error("error executing redis command: {0}")]
    RedisCMDError(redis::RedisError)
}

#[derive(Error, Debug)]
pub enum MobcError {
    #[error("could not get a redis connection from pool: {0}")]
    RedisPoolError(mobc::Error<mobc_redis::redis::RedisError>),
    #[error("error parsing string from redis result: {0}")]
    RedisTypeError(mobc_redis::redis::RedisError),
    #[error("error creating Redis client: {0}")]
    RedisClientError(mobc_redis::redis::RedisError),
    #[error("error executing redis command: {0}")]
    RedisCMDError(mobc_redis::redis::RedisError)
}

impl warp::reject::Reject for Error{}
