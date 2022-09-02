use serde_derive::Deserialize;

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
