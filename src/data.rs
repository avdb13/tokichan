use chrono::{DateTime, Utc};

pub struct Post {
    pub id: u32,
    pub parent: u32,
    pub board: String,
    pub children: Vec<u32>,
    pub created: DateTime::<Utc>,
    pub fields: Fields,
}

#[derive(Default)]
pub struct Fields {
    pub op: String,
    pub email: String,
    pub subject: String,
    pub body: String,
    pub files: Vec<String>,
}

pub struct User {
    pub id: u32,
    pub name: String,
    pub role: u8,
    pub password: String,
    pub created: DateTime::<Utc>,
}
