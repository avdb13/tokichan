use anyhow::Result;
use axum_extra::extract::cookie::Key;
use axum_extra::extract::SignedCookieJar;
use tokio::sync::Mutex;

use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;

use toml::{de::Error, from_str};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use utils::config::Config;
use utils::data::Board;
use utils::models::PoolModel;
use utils::psql::open_db;
use utils::routes::routes;

use crate::utils::fake::populate_db;

mod utils;

pub struct App {
    models: PoolModel,
    config: Config,
    // support hot-reloading boards in the future
    boards: Vec<Board>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // start tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "tokichan-rs=debug,tower_http=debug".into()),
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
    // populate_db(pool.clone()).await?;

    let models = PoolModel { pool: pool.clone() };
    let boards = models.get_boards().await;

    let app = Arc::new(App {
        models,
        config,
        boards,
    });

    let router = routes(app);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}

async fn read_config() -> Result<Config, Error> {
    let s = fs::read_to_string("./tokichan.toml").expect("error reading configuration file");

    from_str(s.as_str())
}
