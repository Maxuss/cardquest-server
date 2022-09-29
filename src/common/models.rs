use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserRegStage {
    pub id: Uuid,
    pub hash: String
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub card_hash: String,
    pub id: Uuid,
    pub username: String
}