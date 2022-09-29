mod handlers;
pub mod models;

use crate::server::handlers::*;
use crate::ServerConfig;
use axum::http::{StatusCode, Uri};
use axum::routing::{get, post};
use axum::{Extension, Router};
use sqlx::PgPool;
use std::net::SocketAddr;
use std::str::FromStr;

#[allow(unused_variables)]
pub async fn init_server(cfg: &ServerConfig, pool: PgPool) -> anyhow::Result<()> {
    let addr = SocketAddr::from_str(&format!("{}:{}", cfg.api.host, cfg.api.port))?;
    log::info!("Starting HTTP server on {}", addr);

    let app = Router::new()
        .route("/user/get/:id", get(get_user))
        .route("/user/register/:sha", post(begin_registration))
        .fallback(handler404)
        .layer(Extension(pool));

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
