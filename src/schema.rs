// @generated automatically by Diesel CLI.

diesel::table! {
    messages (id) {
        id -> Integer,
        room -> Varchar,
        username -> Varchar,
        message -> Varchar,
        file -> Nullable<Varchar>,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        username -> Varchar,
        password -> Varchar,
        token -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    messages,
    users,
);
