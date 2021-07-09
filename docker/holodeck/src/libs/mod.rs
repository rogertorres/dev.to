pub mod models {
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

    #[derive(Debug, Deserialize, Serialize)]
    pub struct NewName{ pub name: String }

    impl PartialEq for Simulation{
        //https://doc.rust-lang.org/std/cmp/trait.Eq.html
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
pub mod filters{
    use warp::Filter;
    use super::{handlers, models};

    fn json_body() -> impl Filter<Extract = (models::Simulation,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }

    fn json_body_put() -> impl Filter<Extract = (models::NewName,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }

    pub fn list_sims(db: models::Db) ->  impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let db_map = warp::any()
            .map(move || db.clone());

        let opt = warp::path::param::<u64>()
            .map(Some)
            .or_else(|_| async { 
                // Ok(None) 
                Ok::<(Option<u64>,), std::convert::Infallible>((None,))
            });

        warp::path!("holodeck" / ..)
            .and(opt)
            .and(warp::path::end())
            .and(warp::get())
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

    pub fn update_sim(db: models::Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let db_map = warp::any()
            .map(move || db.clone());

        warp::path!("holodeck" / u64)
            .and(warp::put())
            .and(json_body_put())
            .and(db_map)
            .and_then(handlers::handle_update_sim)
    }

    pub fn delete_sim(db: models::Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let db_map = warp::any()
            .map(move || db.clone());

        warp::path!("holodeck" / u64)
            .and(warp::delete())
            .and(db_map)
            .and_then(handlers::handle_delete_sim)
    }
}

#[allow(dead_code)]
mod handlers{
    use warp::{http::StatusCode};
    use std::convert::Infallible;
    use crate::libs::models::Simulation;

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

        // if let Some(result) = map.get(&sim){
        if let Some(result) = models::get_simulation(&*map, sim.id){ //0
            return Ok(warp::reply::with_status(
                format!("Simulation #{} already exists under the name {}\n", result.id, result.name), 
                StatusCode::BAD_REQUEST,
            ));
        }

        map.insert(sim.clone());
        Ok(warp::reply::with_status(format!("Simulation #{} created.\n", sim.id), StatusCode::CREATED))
    }

    pub async fn handle_update_sim(id: u64, new: models::NewName, db: models::Db) -> Result<impl warp::Reply, Infallible> {
        // Replaced entry
        if let Some(_) = db.lock().await.replace(Simulation{id, name: new.name}){
            return Ok(warp::reply::with_status(
                format!("Simulation #{} was updated.\n", id), 
                StatusCode::OK,
            ));
        }
        
        // Create entry
        Ok(warp::reply::with_status(
            format!("Simulation #{} was inserted.\n", id), 
            StatusCode::CREATED,
        ))
    }

    pub async fn handle_delete_sim(id: u64, db: models::Db) -> Result<impl warp::Reply, Infallible> {
        if db.lock().await.remove(&Simulation{id, name: String::new(),}){
            return Ok(warp::reply::with_status(
                format!("Simulation #{} was deleted.\n", id), 
                StatusCode::OK,
            ))
        };
        
        Ok(warp::reply::with_status(
            format!("No data was deleted.\n"),
            StatusCode::OK,
        ))
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
    
    #[tokio::test]
    async fn try_update() {
        let db = models::new_db();
        let api = filters::update_sim(db);

        let response = request()
            .method("PUT")
            .path("/holodeck/1")
            .json(&models::NewName{ name: String::from("The Big Goodbye!")})
            .reply(&api)
            .await;

        assert_eq!(response.status(), StatusCode::CREATED);

        let response = request()
            .method("PUT")
            .path("/holodeck/1")
            .json(&models::NewName{ name: String::from("The Short Hello!")})
            .reply(&api)
            .await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn try_delete() {
        let simulation = models::Simulation{
            id: 1, 
            name: String::from("The Big Goodbye!"),
        };

        let db = models::new_db();
        db.lock().await.insert(simulation);

        let api = filters::delete_sim(db);

        let response = request()
            .method("DELETE")
            .path("/holodeck/1")
            .reply(&api)
            .await;

            assert_eq!(response.status(), StatusCode::OK);
    }
}