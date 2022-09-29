use crate::server::handlers::ServerError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Maybe<T> {
    Success {
        success: bool,
        #[serde(flatten)]
        value: T,
    },
    Failure {
        success: bool,
        error: ServerError,
    },
}

impl<T> Maybe<T> {
    pub fn success(value: T) -> Self {
        Self::Success {
            success: true,
            value,
        }
    }

    pub fn failure(error: ServerError) -> Self {
        Self::Failure {
            success: false,
            error,
        }
    }
}

impl<T> IntoResponse for Maybe<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let mut json_resp = Json::into_response(Json(self));
        *json_resp.status_mut() = StatusCode::BAD_REQUEST;
        json_resp
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UserData {
    pub username: String,
    pub card_hash: String,
    pub uuid: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeginRegistration {
    pub card_sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegistrationResponse {
    pub token: String,
    pub bot_url: String,
}
