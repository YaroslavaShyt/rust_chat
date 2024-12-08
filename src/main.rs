#[macro_use]
extern crate diesel;
use crate::auth::User;
use diesel::associations::HasTable;
use diesel::{prelude::*, MysqlConnection};
use rocket::form::Form;
use rocket::fs::FileServer;
use rocket::futures::{SinkExt, StreamExt};
use rocket::response::status::Unauthorized;
use rocket::response::stream::{Event, EventStream};
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::error::RecvError;
use rocket::tokio::sync::broadcast::{channel, Sender};
use rocket::{get, launch, post, routes, Config, FromForm, Shutdown, State};
use std::env;
use rocket_ws::WebSocket;
use tokio_tungstenite::connect_async;

pub mod models;
pub mod schema;

pub mod auth;

use crate::models::{Messages, NewMessage, Users};

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Message {
    pub room: String,
    pub username: String,
    pub message: String,
    pub file: Option<String>,
}

#[post("/message", data = "<form>")]
fn post(
    form: Form<Message>,
    jar: &rocket::http::CookieJar<'_>,
    queue: &State<Sender<Message>>,
) -> Result<(), Unauthorized<String>> {
    if let Some(token_cookie) = jar.get("jwt_token") {
        if let Ok(claims) = auth::validate_jwt(token_cookie.value()) {
            let file = form.file.as_deref().unwrap_or(""); 

            let new_message = NewMessage {
                room: &form.room,
                username: &claims.username,
                message: &form.message,
                file, 
            };

            let conn = establish_connection();
            diesel::insert_into(schema::messages::table)
                .values(new_message)
                .execute(&conn)
                .expect("Error saving new message");

            let _res = queue.send(form.into_inner());
            return Ok(());
        }
    }
    Err(Unauthorized("Unauthorized access".parse().unwrap()))
}



#[get("/events")]
async fn events(queue: &State<Sender<Message>>, mut end: Shutdown) -> EventStream![] {
    let mut rx = queue.subscribe();

    EventStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };
            yield Event::json(&msg);
        }
    }
}


#[get("/chat")]
pub async fn chat_page() -> rocket::fs::NamedFile {
    rocket::fs::NamedFile::open("./static/chat.html")
        .await
        .expect("Unable to find chat.html")
}

#[get("/messages")]
async fn get_messages(
    jar: &rocket::http::CookieJar<'_>,
) -> Result<Json<Vec<Message>>, Unauthorized<String>> {
    if let Some(token_cookie) = jar.get("jwt_token") {
        if let Ok(claims) = auth::validate_jwt(token_cookie.value()) {
            let conn = establish_connection();
            use self::schema::messages::dsl::*;
            let mes: Vec<Messages> = messages::table()
                .load::<Messages>(&conn)
                .expect("Error loading messages");

            let messag: Vec<Message> = mes
                .into_iter()
                .map(|m| Message {
                    room: m.room,
                    username: m.username,
                    message: m.message,
                    file: m.file
                })
                .collect();

            return Ok(Json(messag));
        }
    }
    Err(Unauthorized("Unauthorized access".parse().unwrap()))
}

#[get("/user")]
async fn get_user(jar: &rocket::http::CookieJar<'_>) -> Result<Json<Users>, Unauthorized<String>> {
    if let Some(token_cookie) = jar.get("jwt_token") {
        if let Ok(claims) = auth::validate_jwt(token_cookie.value()) {
            let conn = establish_connection();
            use self::schema::users::dsl::*;
            if let Some(user_data) = users
                .filter(username.eq(&claims.username))
                .first::<Users>(&conn)
                .ok()
            {
                println!("{}", user_data.username);
                return Ok(Json(user_data));
            } else {
                return Err(Unauthorized("User not found".parse().unwrap()));
            }
        }
    }
    Err(Unauthorized("Unauthorized access".parse().unwrap()))
}

#[launch]
fn rocket() -> _ {
    let (sender, _) = channel::<Message>(1024);

    rocket::build()
        .manage(sender)
        .mount("/", FileServer::from("./static"))
        .mount(
            "/",
            routes![
                auth::login_page,
                auth::login,
                auth::register,
                post,
                events,
                get_messages,
                chat_page,
                get_user,
            ],
        )
    }

pub fn establish_connection() -> MysqlConnection {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}
