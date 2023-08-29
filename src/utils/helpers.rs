use std::{fs::read_to_string, time::Duration};

use axum_server::Handle;
use base64::{engine::general_purpose, Engine};
use chrono::Datelike;
use color_eyre::Report;
use digest::Digest;
use ripemd::Ripemd160;
use sqlx::{Pool, Postgres};
use tokio::{fs::File, io::AsyncWriteExt};
use tracing::info;

use super::config::Config;

fn current_year() -> u32 {
    chrono::Utc::now().year() as u32
}

pub fn format_name(name: (&[u8], String)) -> String {
    let mut hasher = Ripemd160::new();
    hasher.update(name.0);

    let base = general_purpose::URL_SAFE_NO_PAD.encode(hasher.finalize().as_slice());
    base + "." + &name.1
}

pub async fn save_file(name: String, bytes: &[u8]) -> Result<(), Report> {
    info!("saving {} with length {} ...", name, bytes.len());

    let mut file = File::create("./.tmp/".to_owned() + &name).await?;
    file.write_all(bytes).await?;

    Ok(())
}

pub async fn hash(input: &[u8]) -> Vec<u8> {
    let mut hasher = Ripemd160::new();
    hasher.update(&input);
    hasher.finalize().as_slice().to_owned()
}

pub async fn read_config() -> Result<Config, toml::de::Error> {
    let s = read_to_string("./tokichan.toml").expect("error reading configuration file");

    toml::from_str(s.as_str())
}

pub async fn graceful_shutdown(handle: Handle, pool: Pool<Postgres>) {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to register shutdown event");

    tracing::info!("shutting down gracefully ...");

    handle.graceful_shutdown(Some(Duration::from_secs(30)));

    pool.close().await;

    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;

        tracing::info!("alive connections: {}", handle.connection_count());
    }
}
