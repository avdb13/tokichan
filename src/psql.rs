use sqlx::{postgres::PgPool, Error};

pub async fn open_db(dsn: &str) -> Result<PgPool, Error> {
    PgPool::connect(dsn).await
}
