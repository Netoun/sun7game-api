#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
#[macro_use(bson, doc)]
extern crate bson;
extern crate mongodb;

use mongodb::db::ThreadedDatabase;
use mongodb::{Client, CommandType, ThreadedClient};

use rocket_contrib::{Json, Value};
use std::env;
use std::sync::Arc;

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

fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/score", routes![get_scores, record_score])
}

fn main() {
    rocket().launch();
}
