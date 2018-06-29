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
    let client = Client::connect(
        &env::var("MONGO_URL").unwrap(),
        env::var("MONGO_PORT").unwrap().parse::<u16>().unwrap(),
    ).unwrap();
    let db = client.db(&env::var("USER").unwrap());
    db.auth(&env::var("USER").unwrap(), &env::var("PASSWORD").unwrap())
        .unwrap();
    let array = db.collection("score");
}

#[post("/", format = "application/json", data = "<score>")]
fn record_score(score: Json<Score>) -> String {
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
fn get_scores() -> Json<Value> {
    let coll = connect_db();
    let mut cursor = coll.find(None, None).ok().expect("Failed to execute find.");
    let docs: Vec<_> = cursor.map(|doc| doc.unwrap()).collect();
    return Json(json!(docs));
}

pub fn options() -> rocket_cors::Cors {
    rocket_cors::Cors {
        allowed_origins: AllowedOrigins::some(&["https://sun7game.netlify.com/"]),
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
        .mount("/score", routes![get_scores, record_score])
}

fn main() {
    rocket().launch();
}
