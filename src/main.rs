use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use toml::{
    from_str,
    de::Error,
};
use std::net::SocketAddr;
use crate::routes::init_app;
use crate::psql::open_db;
use crate::fake::populate_db;
use crate::data::Post;
use std::fs;
use serde_derive::Deserialize;

pub mod routes;
pub mod handlers;
pub mod templates;
pub mod data;
pub mod psql;
pub mod fake;

#[derive(Deserialize)]
pub struct Config {
    pub psql: Psql,
}

#[derive(Deserialize)]
pub struct Psql {
    pub username: String,
    pub password: String,
    pub address: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "example_form=debug".into()))
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = read_config()
        .await
        .expect("error parsing configuration file");
    let dsn = format!("postgresql://{}:{}@{}",
        config.psql.username,
        config.psql.password,
        config.psql.address,
    );


    let client = open_db(dsn.as_str()).await.expect("failed to connect to database");
    populate_db(&client).await;
    let app = init_app();
    let addr = SocketAddr::from(([127,0,0,1], 8080));

    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}


async fn read_config() -> Result<Config, Error> {
    let s = fs::read_to_string("./tokichan.toml")
        .expect("error reading configuration file");

    from_str(s.as_str())
}
