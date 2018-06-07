#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_cors;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
#[macro_use(bson, doc)]
extern crate bson;
extern crate mongodb;

use rocket::http::Method;
use rocket::response::Responder;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, Guard};

use mongodb::db::ThreadedDatabase;
use mongodb::{Client, CommandType, ThreadedClient};

use rocket_contrib::{Json, Value};
use std::env;

#[derive(Serialize, Deserialize)]
pub struct Score {
    pub name: String,
    pub point: u32,
}

fn connect_db() -> mongodb::coll::Collection {
    let client = Client::connect(
        &env::var("MONGO_URL").unwrap(),
        env::var("MONGO_PORT").unwrap().parse::<u16>().unwrap(),
    ).unwrap();
    let db = client.db(&env::var("USER").unwrap());
    db.auth(&env::var("USER").unwrap(), &env::var("PASSWORD").unwrap())
        .unwrap();
    return db.collection("score");
}

#[post("/", format = "application/json", data = "<score>")]
fn record_score<'a>(score: Json<Score>) -> String {
    let coll = connect_db();
    let name = &score.name;
    let point = score.point;
    let doc = doc! {
        "name": name,
        "point": point
    };
    match coll.insert_one(doc, None) {
        Ok(_) => "Done".to_string(),
        Err(e) => e.to_string(),
    }
}

#[get("/", format = "application/json")]
fn get_scores<'a>() -> Json<Value> {
    let coll = connect_db();
    let mut cursor = coll.find(None, None).ok().expect("Failed to execute find.");
    let docs: Vec<_> = cursor.map(|doc| doc.unwrap()).collect();
    return Json(json!(docs));
}

#[options("/")]
fn ping_options<'r>() -> impl Responder<'r> {
    let options = cors_options_all();
    options.respond_owned(|guard| guard.responder(()))
}

fn cors_options() -> Cors {
    let (allowed_origins, failed_origins) = AllowedOrigins::some(&["*"]);
    assert!(failed_origins.is_empty());

    // You can also deserialize this
    rocket_cors::Cors {
        allowed_origins: allowed_origins,
        ..Default::default()
    }
}

fn cors_options_all() -> Cors {
    // You can also deserialize this
    Default::default()
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/score", routes![get_scores, record_score])
        .mount("/score", rocket_cors::catch_all_options_routes())
        .manage(cors_options())
}

fn main() {
    rocket().launch();
}
