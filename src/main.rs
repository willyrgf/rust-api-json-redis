use warp::Filter;

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
