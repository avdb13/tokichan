// i32 is used over u32 because this is a requirement by `sqlx` despite the types never being
// negative
use chrono::{DateTime, Utc};

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

#[derive(Clone)]
pub struct Board {
    pub name: String,
    pub title: String,
}

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

pub struct User {
    pub id: i32,
    pub name: String,
    pub role: i8,
    pub password: String,
    pub created: DateTime<Utc>,
}
