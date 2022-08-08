use std::sync::Mutex;
use sqlx::{
    Error,
    Connection,
    postgres::{
        PgConnection,
    },
};

pub async fn open_db(dsn: &str) -> Result<Mutex<PgConnection>, Error> {
    match PgConnection::connect(dsn).await {
        Ok(conn) => Mutex::new(conn),
        Err(e) => e,
    }
}
