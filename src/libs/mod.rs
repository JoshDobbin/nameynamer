pub mod models {
    use serde::{Deserialize, Serialize};
    use std::collections::HashSet;
    use std::hash::{Hash, Hasher};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Name {
        pub name: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct NewName{ pub name: String }

    impl PartialEq for Name{
        //https://doc.rust-lang.org/std/cmp/trait.Eq.html
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name
        }
    }
    
    impl Eq for Name {}
    
    impl Hash for Name{
        fn hash<H: Hasher>(&self, state: &mut H){
                self.name.hash(state);
        }
    }

    pub fn get_name<'a>(names: &'a HashSet<Name>, name: String) -> Option<&'a Name>{
        names.get(&Name{
            name: name.clone(),
        })
    }

    pub type Db = Arc<Mutex<HashSet<Name>>>;

    #[allow(dead_code)]
    pub fn new_db() -> Db {
        Arc::new(Mutex::new(HashSet::new()))
    }
}

#[allow(dead_code)]
pub mod filters{
    use warp::Filter;
    use super::{handlers, models};

    fn json_body() -> impl Filter<Extract = (models::Name,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }

    fn json_body_put() -> impl Filter<Extract = (models::NewName,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }

    pub fn list_names(db: models::Db) ->  impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let db_map = warp::any()
            .map(move || db.clone());

        warp::path!("list")
            .and(warp::path::end())
            .and(warp::get())
            .and(db_map)
            .and_then(handlers::handle_list_names)
    }

    pub fn post_hello(db: models::Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let db_map = warp::any()
            .map(move || db.clone());

        warp::path!("hello" / String)
            .and(db_map)
            .and_then(handlers::handle_create_name)
    }
}

#[allow(dead_code)]
mod handlers{
    use warp::{http::StatusCode};
    use std::convert::Infallible;
    use crate::libs::models::Name;

    use super::models;

    pub async fn handle_list_names(db: models::Db) -> Result<impl warp::Reply, Infallible> {
        let result = db.lock().await.clone();
        Ok(warp::reply::json(&result)) 
    }

    pub async fn handle_create_name(name: String, db: models::Db) -> Result<impl warp::Reply, Infallible> {
        let mut map = db.lock().await;

        // if let Some(result) = map.get(&sim){
        if let Some(result) = models::get_name(&*map, name.clone()){ //0
            return Ok(warp::reply::with_status(
                format!("Name {} already exists\n", result.name), 
                StatusCode::BAD_REQUEST,
            ));
        }

        let name_obj = Name { name: name.clone() };
        map.insert(name_obj);
        Ok(warp::reply::with_status(format!("Name {} created.\n", name), StatusCode::CREATED))
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

        let simulation1 = models::Name{
            name: String::from("Doug"),
        };


        let simulation2 = models::Name{
            name: String::from("Tim"),
        };

        let db = models::new_db();
        db.lock().await.insert(simulation1.clone());
        db.lock().await.insert(simulation2.clone());

        let api = filters::list_names(db);

        let response = request()
            .method("GET")
            .path("/list")
            .reply(&api)
            .await;

        let result: Vec<u8> = response.into_body().into_iter().collect();
        let result = str::from_utf8(&result).unwrap();
        let result: HashSet<models::Name> = serde_json::from_str(result).unwrap();
        assert_eq!(models::get_name(&result, simulation1.name.clone()).unwrap(), &simulation1);
        assert_eq!(models::get_name(&result, simulation2.name.clone()).unwrap(), &simulation2);
    }

    #[tokio::test]
    async fn try_create() {
        let db = models::new_db();
        let api = filters::post_hello(db);
    
        let response = request()
            .method("POST")
            .path("/hello/Ray")
            .reply(&api)
            .await;
    
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn try_create_duplicates() {
        let db = models::new_db();
        let api = filters::post_hello(db);
    
        let response = request()
            .method("POST")
            .path("/hello/Steve")
            .reply(&api)
            .await;
    
        assert_eq!(response.status(), StatusCode::CREATED);
    
        let response = request()
            .method("POST")
            .path("/hello/Steve")
            .reply(&api)
            .await;
    
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

}