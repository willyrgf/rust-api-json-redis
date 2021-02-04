use std::convert::Infallible;
use warp::{Filter, Rejection, Reply};
use thiserror::Error;

mod direct;

type WebResult<T> = std::result::Result<T, Rejection>;
type Result<T> = std::result::Result<T, Error>;

const REDIS_CON_STRING: &str = "redis://127.0.0.1";

#[tokio::main]
async fn main() {
    let redis_client = redis::Client::open(REDIS_CON_STRING)
        .expect("can create redis client");

    let direct_route = warp::path!("direct")
        .and(with_redis_client(redis_client.clone()))
        .and_then(direct_handler);

    let routes = direct_route;

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;

}

fn with_redis_client(
    client: redis::Client
) -> impl Filter<Extract = (redis::Client,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
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

#[derive(Error, Debug)]
pub enum Error {
    #[error("direct redis error: {0}")]
    DirectError(#[from] DirectError),
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

impl warp::reject::Reject for Error{}
