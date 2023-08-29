use crate::utils::{
    captcha::CaptchaService,
    data::App,
    fake::ImagePool,
    helpers::{graceful_shutdown, read_config},
};

use axum_server::Handle;
use color_eyre::eyre::Result;
use color_eyre::Report;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use utils::{models::PoolModel, psql::open_db, routes::routes};

mod utils;

#[tokio::main]
async fn main() -> Result<(), Report> {
    color_eyre::install()?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "tokichan=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = read_config()
        .await
        .expect("error parsing configuration file");
    let dsn = format!(
        "postgresql://{}:{}@{}",
        config.psql.username, config.psql.password, config.psql.address,
    );
    let pool = open_db(dsn.as_str())
        .await
        .expect("failed to connect to database");

    let models = PoolModel { pool: pool.clone() };
    let boards = models.get_boards().await;

    let app = Arc::new(App::new(config, models, boards));
    let cs = Arc::new(RwLock::new(CaptchaService::new(10).await));
    let router = routes(app, cs);

    let handle = Handle::new();
    tokio::spawn(graceful_shutdown(handle.clone(), pool.clone()));

    let mut ip = ImagePool::new(&pool, 20).await?;
    ip.truncate().await?;
    ip.mock(10, 5).await?;

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    tracing::debug!("listening on {}", addr);

    axum_server::bind(addr)
        .handle(handle)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}
