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
#[macro_use]
extern crate log;

use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins};

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
    info!("connection start");
    let client = Client::connect(
        &env::var("MONGO_URL").unwrap(),
        env::var("MONGO_PORT").unwrap().parse::<u16>().unwrap(),
    ).unwrap();
    let db = client.db(&env::var("USER").unwrap());
    db.auth(&env::var("USER").unwrap(), &env::var("PASSWORD").unwrap())
        .unwrap();
    info!("db : {:?}", &env::var("MONGO_URL"));
    return db.collection("score");
}

#[post("/score", format = "application/json", data = "<score>")]
fn record_score(score: Json<Score>) -> String {
    info!("POST score");
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

#[get("/score", format = "application/json")]
fn get_scores() -> Json<Value> {
    info!("GET score");
    let coll = connect_db();
    let mut cursor = coll.find(None, None).ok().expect("Failed to execute find.");
    let docs: Vec<_> = cursor.map(|doc| doc.unwrap()).collect();
    return Json(json!(docs));
}

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

pub fn options() -> rocket_cors::Cors {
    rocket_cors::Cors {
        allowed_origins: AllowedOrigins::all(),
        allowed_methods: vec![Method::Post, Method::Get]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept", "Content-Type"]),
        allow_credentials: true,
        ..Default::default()
    }
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .attach(options())
        .mount("/", routes![hello, get_scores, record_score])
}

fn main() {
    pretty_env_logger::init();
    rocket().launch();
}
