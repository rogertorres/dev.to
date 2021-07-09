use warp::Filter;
mod libs;

#[tokio::main]
async fn main() {
    use libs::{filters, models};

    let db = models::new_db();

    let routes = filters::list_sims(db.clone())
        .or(filters::post_sim(db.clone()))
        .or(filters::update_sim(db.clone()))
        .or(filters::delete_sim(db.clone()));

    println!("Warp 6, Engage!");
    warp::serve(routes)
        .run(([0, 0, 0, 0], 3030))
        .await;
}
