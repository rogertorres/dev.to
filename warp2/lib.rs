mod models {
    use serde::{Deserialize, Serialize};
    use std::collections::HashSet;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
    pub struct Simulation {
        pub id: u64,
        pub name: String,
    }

    pub type Db = Arc<Mutex<HashSet<Simulation>>>;

    pub fn new_db() -> Db {
        Arc::new(Mutex::new(HashSet::new()))
    }
}

mod filters{
    use warp::Filter;
    use super::{handlers, models};

    fn json_body() -> impl Filter<Extract = (models::Simulation,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }

    pub fn list_sims() ->  impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone{ 
        warp::path!("holodeck")
            .and(warp::get())
            .and_then(handlers::handle_list_sims)
    }

    pub fn post_sim(db: models::Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let db_map = warp::any()
            .map(move || db.clone());

        warp::path!("holodeck")
            .and(warp::post())
            .and(json_body())
            .and(db_map)
            .and_then(handlers::handle_create_sim)
    }
}

mod handlers{
    use warp::http::StatusCode;
    use std::convert::Infallible;
    use super::models;

    pub async fn handle_list_sims() -> Result<impl warp::Reply, Infallible> {
        // "Alright, alright, alright", Matthew said.
        Ok(StatusCode::ACCEPTED)
    }

pub async fn handle_create_sim(sim: models::Simulation, db: models::Db) -> Result<impl warp::Reply, Infallible> {
    let mut map = db.lock().await;

    if let Some(result) = map.get(&sim){
        return Ok(warp::reply::with_status(
            format!("Simulation #{} already exists under the name {}", result.id, result.name), 
            StatusCode::BAD_REQUEST,
        ));
    }

    map.insert(sim.clone());
    Ok(warp::reply::with_status(format!("Simulation #{} created", sim.id), StatusCode::CREATED))
}
}

#[cfg(test)]
mod tests {
    use warp::http::StatusCode;
    use warp::test::request;
    use super::{filters,models};

    #[tokio::test]
    async fn try_list() {
        let api = filters::list_sims();

        let response = request()
            .method("GET")
            .path("/holodeck")
            .reply(&api)
            .await;

        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn try_create() {
        let db = models::new_db();
        let api = filters::post_sim(db);
    
        let response = request()
            .method("POST")
            .path("/holodeck")
            .json(&models::Simulation{
                id: 1,
                name: String::from("The Big Goodbye")
            })
            .reply(&api)
            .await;
    
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn try_create_duplicates() {
        let db = models::new_db();
        let api = filters::post_sim(db);
    
        let response = request()
            .method("POST")
            .path("/holodeck")
            .json(&models::Simulation{
                id: 1,
                name: String::from("Bride Of Chaotica!")
            })
            .reply(&api)
            .await;
    
        assert_eq!(response.status(), StatusCode::CREATED);
    
        let response = request()
            .method("POST")
            .path("/holodeck")
            .json(&models::Simulation{
                id: 1,
                name: String::from("Bride Of Chaotica!")
            })
            .reply(&api)
            .await;
    
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}