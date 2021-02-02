use warp::{Filter, Rejection};
use thiserror::Error;

mod direct;

type WebResult<T> = std::result::Result<T, Rejection>;
type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() {
    let hello = warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name));

    let plus = warp::path!("plus" / f64 / f64 )
        .map(|n1, n2| format!("plus: {}", n1+n2));

    let routes = hello.or(plus);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;

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
}
