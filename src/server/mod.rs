use crate::ServerConfig;
use sqlx::PgPool;
use std::net::SocketAddr;
use std::str::FromStr;

#[allow(unused_variables)]
pub async fn init_server(cfg: &ServerConfig, pool: PgPool) -> anyhow::Result<()> {
    let addr = SocketAddr::from_str(&format!("{}:{}", cfg.api.host, cfg.api.port))?;
    log::info!("Starting diary server on {}", addr);

    Ok(())
}
