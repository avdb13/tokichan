use crate::config::Config;
use crate::fake::populate_db;
use crate::psql::open_db;
use crate::routes::routes;
use models::PoolModel;
use std::fs;
use std::net::SocketAddr;
use std::sync::Mutex;
use toml::{de::Error, from_str};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod config;
pub mod data;
pub mod fake;
pub mod handlers;
pub mod models;
pub mod psql;
pub mod routes;
pub mod templates;

use axum::Extension;

pub struct App {
    models: PoolModel,
    config: Config,
}

fn init_app() {}

#[tokio::main]
async fn main() {
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
    populate_db(pool).await;
    let app = Arc::new(App {
        models: PoolModel { pool },
        config,
    });
    let router = routes(Extension(app));
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

async fn read_config() -> Result<Config, Error> {
    let s = fs::read_to_string("./tokichan.toml").expect("error reading configuration file");

    from_str(s.as_str())
}
