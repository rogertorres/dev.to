use warp::Filter;
use holodeck::{ models, filters };

#[tokio::main]
async fn main() {

    let db = models::new_db();

    let routes = filters::list_sims(db.clone())
        .or(filters::post_sim(db.clone()))
        .or(filters::update_sim(db.clone()))
        .or(filters::delete_sim(db.clone()));

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
