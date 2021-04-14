mod models {
    use serde::{Deserialize, Serialize};
    use std::collections::HashSet;
    use std::hash::{Hash, Hasher};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Simulation {
        pub id: u64,
        pub name: String,
    }

    impl PartialEq for Simulation{
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }
    
    impl Eq for Simulation {}
    
    impl Hash for Simulation{
        fn hash<H: Hasher>(&self, state: &mut H){
                self.id.hash(state);
        }
    }

    pub fn get_simulation<'a>(sims: &'a HashSet<Simulation>, id: u64) -> Option<&'a Simulation>{
        sims.get(&Simulation{
            id,
            name: String::new(),
        })
    }

    pub type Db = Arc<Mutex<HashSet<Simulation>>>;

    #[allow(dead_code)]
    pub fn new_db() -> Db {
        Arc::new(Mutex::new(HashSet::new()))
    }
}

#[allow(dead_code)]
mod filters{
    use warp::Filter;
    use super::{handlers, models};

    fn json_body() -> impl Filter<Extract = (models::Simulation,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }

    pub fn list_sims(db: models::Db) ->  impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let db_map = warp::any()
            .map(move || db.clone());

        let opt = warp::path::param::<u64>()
            .map(Some)
            .or_else(|_| async { 
                Ok::<(Option<u64>,), std::convert::Infallible>((None,))
            });

        warp::path!("holodeck" / ..)
            .and(opt)
            .and(warp::path::end())
            .and(db_map)
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

#[allow(dead_code)]
mod handlers{
    use warp::{http::StatusCode};
    use std::convert::Infallible;
    use super::models;

    pub async fn handle_list_sims(opt: Option<u64>, db: models::Db) -> Result<impl warp::Reply, Infallible> {
        let mut result = db.lock().await.clone();
        if let Some(param) = opt {
            result.retain(|k| k.id == param);
        }
        Ok(warp::reply::json(&result)) 
    }

    pub async fn handle_create_sim(sim: models::Simulation, db: models::Db) -> Result<impl warp::Reply, Infallible> {
        let mut map = db.lock().await;

        if let Some(result) = models::get_simulation(&*map, sim.id){
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
    use std::collections::HashSet;

    #[tokio::test]
    async fn try_list() {
        use std::str;
        use serde_json;

        let simulation1 = models::Simulation{
            id: 1, 
            name: String::from("The Big Goodbye!"),
        };


        let simulation2 = models::Simulation{
            id: 2, 
            name: String::from("Bride Of Chaotica!"),
        };

        let db = models::new_db();
        db.lock().await.insert(simulation1.clone());
        db.lock().await.insert(simulation2.clone());

        let api = filters::list_sims(db);

        let response = request()
            .method("GET")
            .path("/holodeck")
            .reply(&api)
            .await;

        let result: Vec<u8> = response.into_body().into_iter().collect();
        let result = str::from_utf8(&result).unwrap();
        let result: HashSet<models::Simulation> = serde_json::from_str(result).unwrap();
        assert_eq!(models::get_simulation(&result, 1).unwrap(), &simulation1);
        assert_eq!(models::get_simulation(&result, 2).unwrap(), &simulation2);

    let response = request()
        .method("GET")
        .path("/holodeck/2")
        .reply(&api)
        .await;

    let result: Vec<u8> = response.into_body().into_iter().collect();
    let result = str::from_utf8(&result).unwrap();
    let result: HashSet<models::Simulation> = serde_json::from_str(result).unwrap();
    assert_eq!(result.len(),1);
    assert_eq!(models::get_simulation(&result, 2).unwrap(), &simulation2);
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