mod handlers;
pub mod models;

use crate::common::questions::QuizHandler;
use crate::server::handlers::*;
use crate::ServerConfig;
use axum::http::{StatusCode, Uri};
use axum::routing::{get, post};
use axum::{Extension, Router};
use sqlx::PgPool;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

#[allow(unused_variables)]
pub async fn init_server(cfg: &ServerConfig, pool: PgPool) -> anyhow::Result<()> {
    let addr = SocketAddr::from_str(&format!("{}:{}", cfg.api.host, cfg.api.port))?;
    log::info!("Starting HTTP server on {}", addr);

    let quiz = QuizHandler::new("questions");

    let app = Router::new()
        .route("/user/get/id/:id", get(get_user_id))
        .route("/user/get/sha/:hash", get(get_user_sha))
        .route("/user/register/:sha", post(begin_registration))
        .route("/user/:user/question/:category", get(get_question))
        .route("/quiz/answer/:question/:answer", post(answer_question))
        .fallback(handler404)
        .layer(Extension(pool))
        .layer(Extension(Arc::new(Mutex::new(quiz))));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn handler404(path: Uri) -> (StatusCode, Payload<()>) {
    (
        StatusCode::NOT_FOUND,
        err(ServerError::NotFound(format!(
            "Invalid request path: {}",
            path
        ))),
    )
}
