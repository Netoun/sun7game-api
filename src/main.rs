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
use mongodb::{Client, ThreadedClient};
use rocket_contrib::{Json, Value};
use std::env;

#[derive(Serialize, Deserialize)]
pub struct Score {
    pub name: String,
    pub point: u32,
}

#[post("/", format = "application/json", data = "<score>")]
fn record_score(score: Json<Score>) -> &'static str {
    match env::var("MONGODB_URI") {
        Ok(uri) => {
            let client = Client::connect(&uri, 27017)
                .ok()
                .expect("Error establishing connection.");

            // The users collection
            let coll = client.db("game").collection("score");
            let name = &score.name;
            let point = score.point;
            let doc = doc! {
                "name": name,
                "point": point
            };
            match coll.insert_one(doc, None) {
                Ok(_) => "Done",
                Err(e) => "Error",
            }
        }
        Err(e) => "Error mongo",
    }
}

#[get("/", format = "application/json")]
fn get_scores() -> Json<Value> {
    match env::var("MONGODB_URI") {
        Ok(uri) => {
            let client = Client::connect("localhost", 27017)
                .ok()
                .expect("Error establishing connection.");

            // The users collection
            let coll = client.db("game").collection("score");
            let mut cursor = coll.find(None, None).ok().expect("Failed to execute find.");
            let docs: Vec<_> = cursor.map(|doc| doc.unwrap()).collect();
            Json(json!(docs))
        }
        Err(e) => Json(json!({ "error": "error" })),
    }
}

fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/score", routes![get_scores, record_score])
}

fn main() {
    rocket().launch();
}
