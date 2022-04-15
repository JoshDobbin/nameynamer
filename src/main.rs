use warp::Filter;
mod libs;

#[tokio::main]
async fn main() {
    use libs::{filters, models};

    let db = models::new_db();

    let routes = filters::list_names(db.clone())
        .or(filters::post_hello(db.clone()));

    println!("Let's gooooo!");
    warp::serve(routes)
        .run(([0, 0, 0, 0], 3030))
        .await;
}