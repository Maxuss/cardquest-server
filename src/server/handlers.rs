use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use axum_extra::extract::WithRejection;
use std::io;
// use axum_extra::extract::WithRejection;
use crate::common::models::StoredUser;
use crate::server::models::{Maybe, RegistrationResponse, UserData};
use serde::{Serialize, Serializer};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("An unknown error has occurred: `{0}`")]
    Unknown(String),
    #[error("An IO error has occurred: `{0}`")]
    IO(#[from] io::Error),
    #[error("Invalid POST payload: `{0}`")]
    InvalidPostData(#[from] JsonRejection),
    #[error("Invalid data format in path: `{0}`")]
    InvalidPathData(#[from] PathRejection),
    #[error("Data not found: `{0}`")]
    NotFound(String),
    #[error("Could not parse value: `{0}`")]
    ParsingError(#[from] std::fmt::Error),
    #[error("Could not parse UUID: `{0}`")]
    UuidError(#[from] uuid::Error),
    #[error("SQL Database error: `{0}`")]
    SqlError(#[from] sqlx::Error),
    #[error("Invalid SHA hash string provided")]
    ShaError,
    #[error("User with card SHA `{0}` already exists!")]
    UserExists(String),
}

impl Serialize for ServerError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        Maybe::<()>::failure(self).into_response()
    }
}

pub type Payload<T> = axum::response::Result<Json<Maybe<T>>, ServerError>;

pub fn success<T>(value: T) -> Payload<T> {
    Ok(Json(Maybe::success(value)))
}

pub fn err<T>(err: ServerError) -> Payload<T> {
    Ok(Json(Maybe::failure(err)))
}

pub async fn get_user(
    Path(id): Path<Uuid>,
    Extension(pool): Extension<PgPool>,
) -> Payload<UserData> {
    if let Some(user) = sqlx::query_as::<_, StoredUser>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await?
    {
        success(UserData {
            username: user.username,
            card_hash: user.card_hash,
            uuid: user.id,
        })
    } else {
        err(ServerError::NotFound(format!(
            "Could not find user with id `{}` in the database!",
            id
        )))
    }
}

pub async fn begin_registration(
    WithRejection(Path(sha), _): WithRejection<Path<String>, ServerError>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    if sha.len() != 64 {
        return err(ServerError::ShaError);
    }
    if sqlx::query_as::<_, StoredUser>("SELECT * FROM users WHERE card_hash = $1")
        .bind(&sha)
        .fetch_optional(&pool)
        .await?
        .is_some()
    {
        return err(ServerError::UserExists(sha));
    }
    let rows = sqlx::query("INSERT INTO users_reg VALUES ($1, $2)")
        .bind(&sha)
        .bind(Uuid::new_v4())
        .execute(&pool)
        .await?;
    if rows.rows_affected() < 1 {
        log::warn!("Invalid amount of rows affected for user registration process initialization.")
    }
    success(RegistrationResponse {
        token: sha[..8].to_owned(),
        bot_url: "https://t.me/cardquest_bot".to_owned(),
    })
}
