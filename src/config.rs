use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub psql: Psql,
    pub security: Security,
}

#[derive(Deserialize)]
pub struct Psql {
    pub username: String,
    pub password: String,
    pub address: String,
}

#[derive(Deserialize)]
pub struct Security {
    pub upload_limit: String,
    pub allowed_mimes: Vec<String>,
}
