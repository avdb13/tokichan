use sqlx::{
    Error,
    Connection,
    postgres::{
        PgConnection,
    },
};

pub async fn open_db(dsn: &str) -> Result<PgConnection, Error> {
    PgConnection::connect(dsn).await
}
