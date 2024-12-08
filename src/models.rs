use diesel::prelude::*;
use diesel::{Insertable, Queryable};
use rocket::serde::{Serialize};
use crate::schema::{messages, users};

#[derive(Queryable)]
#[derive(Serialize)]
pub struct Users {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub token: String,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password: &'a str,
    pub token: &'a str,
}

#[derive(Queryable)]
#[derive(Serialize)]
pub struct Messages {
    pub id: i32,
    pub room: String,
    pub username: String,
    pub message: String,
    pub file: Option<String>,
}

#[derive(Insertable)]
#[table_name = "messages"]
pub struct NewMessage<'a> {
    pub room: &'a str,
    pub username: &'a str,
    pub message: &'a str,
    pub file: &'a str,
}
