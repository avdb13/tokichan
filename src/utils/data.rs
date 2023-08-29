

// i32 is used over u32 because this is a requirement by `sqlx` despite the types never being
// negative

use chrono::{DateTime, Utc};

use pbkdf2::{
    password_hash::{PasswordHash, PasswordHasher},
    Pbkdf2,
};
use serde::Deserialize;

use super::{config::Config, models::PoolModel};


pub struct App {
    pub models: PoolModel,
    // support hot-reloading boards in the future
    pub config: Config,
    pub boards: Vec<Board>,
}

impl App {
    pub fn new(config: Config, models: PoolModel, boards: Vec<Board>) -> Self {
        Self {
            config,
            models,
            boards,
        }
    }
}

pub struct ValidCaptcha(pub bool);

#[derive(Debug)]
pub struct Post {
    pub id: i32,
    pub parent: Option<i32>,
    pub board: String,
    pub created: DateTime<Utc>,

    pub op: String,
    pub email: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,

    pub files: Option<Vec<String>>,
}

impl Default for Post {
    fn default() -> Self {
        Post {
            id: 0,
            parent: None,
            board: "/b/".to_string(),
            files: None,
            created: Utc::now(),

            op: "Me".to_string(),
            email: None,
            subject: None,
            body: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub enum Role {
    Admin,
    Moderator,
    Volunteer,
    User,
}

#[derive(Default, Debug, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub salt: String,
    pub role: Option<Role>,
}

impl Credentials {
    pub fn hash(&self) -> PasswordHash {
        Pbkdf2
            .hash_password(self.password.as_bytes(), &self.salt)
            .unwrap()
    }

    pub fn authenticate(&self) -> bool {
        // let parsed_hash = PasswordHash::new(&self.hash)?;
        // assert!(Pbkdf2.verify_password(password, &parsed_hash).is_ok());
        false
    }
}

#[derive(Clone)]
pub struct Board {
    pub name: String,
    pub title: String,
}

pub struct User {
    pub id: i32,
    pub name: String,
    pub role: i8,
    pub password: String,
    pub created: DateTime<Utc>,
}
