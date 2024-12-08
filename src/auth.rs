use crate::diesel::ExpressionMethods;
use bcrypt::DEFAULT_COST;
use bcrypt::{hash, verify};
use crate::models::{NewUser, Users};
use crate::{establish_connection, schema};
use diesel::{OptionalExtension, RunQueryDsl};
use jsonwebtoken::errors::Result as JwtResult;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rocket::http::{Cookie, Status};
use rocket::outcome::Outcome;
use rocket::request::FromRequest;
use rocket::response::status::Custom;
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::sync::broadcast::{channel, Sender};
use rocket::{get, post, routes, serde::json::Json, Request};
use diesel::prelude::*;
use rocket::response::Redirect;
use crate::schema::users;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub username: String,
    exp: usize,
}

fn create_jwt(user: &User) -> JwtResult<String> {
    let my_claims = Claims {
        username: user.username.clone(),
        exp: 10000000000,
    };
    let secret = b"secret";
    encode(
        &Header::default(),
        &my_claims,
        &EncodingKey::from_secret(secret),
    )
}

pub(crate) fn validate_jwt(token: &str) -> JwtResult<Claims> {
    let validation = Validation {
        leeway: 0,
        validate_exp: true,
        ..Validation::default()
    };
    let decoding_key = DecodingKey::from_secret("secret".as_ref());
    decode::<Claims>(token.split_whitespace().last().unwrap()
                     , &decoding_key, &validation).map(|data| data.claims)
}

#[post("/login", data = "<user>")]
pub fn login(user: Json<User>, jar: &rocket::http::CookieJar<'_>) -> Result<Json<String>, Custom<String>> {
    let conn = establish_connection();

    let u = schema::users::table
        .filter(schema::users::username.eq(&user.username))
        .first::<Users>(&conn)
        .optional()
        .expect("Error querying database");

    match u {
        Some(db_user) => {
            if verify(&user.password, &db_user.password).unwrap_or(false) {
                let token = create_jwt(&user).unwrap();

                diesel::update(schema::users::table.filter(schema::users::id.eq(db_user.id)))
                    .set(users::token.eq(&token))
                    .execute(&conn)
                    .expect("Error updating token");

                jar.add(Cookie::new("jwt_token", token.clone()));

                Ok(Json(token.to_string()))
            } else {
                Err(Custom(Status::Unauthorized, "Invalid credentials".to_string()))
            }
        },
        None => Err(Custom(Status::Unauthorized, "User not found".to_string())),
    }
}


#[post("/register", data = "<user>")]
pub fn register(user: Json<User>, jar: &rocket::http::CookieJar<'_>) -> Result<Json<String>, Custom<String>> {
    let conn = establish_connection();


    let existing_user = schema::users::table
        .filter(schema::users::username.eq(&user.username))
        .first::<Users>(&conn)
        .optional()
        .expect("Error querying database");

    if existing_user.is_some() {
        return Err(Custom(
            Status::BadRequest,
            "Username already taken".to_string(),
        ));
    }

    let hashed_password = hash(&user.password, DEFAULT_COST).unwrap();

    let token = create_jwt(&user).unwrap();

    let new_user = NewUser {
        username: &user.username.clone(),
        password: &hashed_password,
        token: &token,
    };

    jar.add(Cookie::new("jwt_token", token.clone()));

    diesel::insert_into(schema::users::table)
        .values(&new_user)
        .execute(&conn)
        .expect("Error saving new user");

    Ok(Json("Registration successful".to_string()))
}


#[get("/")]
pub async fn login_page() -> rocket::fs::NamedFile {
    rocket::fs::NamedFile::open("./static/auth.html")
        .await
        .expect("Unable to find auth.html")
}









#[derive(Debug)]
pub struct JwtAuth(pub Claims);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for JwtAuth {
    type Error = String;

    async fn from_request(
        req: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        if let Some(auth_header) = req.headers().get_one("Authorization") {
            println!("Received Authorization header: {}", auth_header);
            match validate_jwt(auth_header) {
                Ok(claims) => Outcome::Success(JwtAuth(claims)),
                Err(_) => Outcome::Error((
                    rocket::http::Status::Unauthorized,
                    "Invalid token".to_string(),
                )),
            }
        } else {
            Outcome::Error((
                rocket::http::Status::Unauthorized,
                "Missing token".to_string(),
            ))
        }
    }
}
