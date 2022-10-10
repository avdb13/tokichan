use crate::{
    data::*,
    templates::{Input, LoginError},
};
use anyhow::Result;
use axum::{extract::Multipart, response::Redirect};
use axum_sessions::extractors::WritableSession;
use crypto::password_hash::PasswordHash;
use mime_sniffer::MimeTypeSniffer;
use pbkdf2::password_hash::PasswordVerifier;
use pbkdf2::Pbkdf2;
use ripemd::{Digest, Ripemd160};
use sqlx::PgPool;
use thiserror::Error;
use tokio::{fs::File, io::AsyncWriteExt};
use tracing::{event, Level};

#[derive(Error, Debug)]
pub enum InputError {
    #[error("file `{0}` doesn't have a recognized type")]
    Files(u32),
}
pub struct PoolModel {
    pub pool: PgPool,
}

impl PoolModel {
    pub async fn get_users(&self) -> Result<Vec<String>> {
        let result = sqlx::query!(
            r#"
             SELECT name from users
        "#
        )
        .fetch_all(&self.pool)
        .await
        .expect("Oops");

        Ok(result.iter().map(|x| x.name.clone()).collect::<Vec<_>>())
    }

    pub async fn recent(&self) -> Vec<Post> {
        sqlx::query_as!(
            Post,
            r#"
             SELECT id, parent, board, created, op, email, body, subject, files FROM posts
        "#
        )
        .fetch_all(&self.pool)
        .await
        .expect("Oops")
    }

    pub async fn get_board(&self, board: String) -> Vec<Post> {
        sqlx::query_as!(
            Post,
            r#"
             SELECT id, parent, board, created, op, email, body, subject, files FROM posts where board = $1
        "#,
            board,
        )
        .fetch_all(&self.pool)
        .await
        .expect("Oops")
    }

    pub async fn signup(&self, credentials: Credentials) -> Result<()> {
        if credentials.username.is_empty() {
            return Err(LoginError::EmptyUsername.into());
        } else if credentials.password.is_empty() {
            return Err(LoginError::EmptyPassword.into());
        }

        let password = credentials.hash().unwrap();
        sqlx::query!(
            r#"
                     INSERT INTO users(name, password, role)
                     VALUES ($1, $2, $3)
                "#,
            credentials.username,
            password,
            format!("{:?}", credentials.role.clone().unwrap()).to_lowercase(),
        )
        .execute(&self.pool)
        .await
        .expect("Oops");
        Ok(())
    }

    pub async fn logout(&self, username: String, mut session: WritableSession) -> Redirect {
        session.remove(&username);
        // Inject flash message
        Redirect::to("/")
    }

    pub async fn login(
        &self,
        credentials: Credentials,
        mut session: WritableSession,
    ) -> Result<Redirect, LoginError> {
        if credentials.username.is_empty() {
            return Err(LoginError::EmptyUsername);
        } else if credentials.password.is_empty() {
            return Err(LoginError::EmptyPassword);
        }

        let result = sqlx::query!(
            r#"
             SELECT password FROM users where name = $1
        "#,
            credentials.username,
        )
        .fetch_optional(&self.pool)
        .await
        .expect("Oops");

        match result {
            None => Err(LoginError::NonExistentUser(credentials.username)),
            Some(record) => {
                let hash = PasswordHash::new(&record.password).unwrap();

                match Pbkdf2.verify_password(credentials.password.as_bytes(), &hash) {
                    Ok(_) => {
                        session
                            .insert(
                                &credentials.username,
                                chrono::Utc::now()
                                    .checked_add_signed(chrono::Duration::hours(1))
                                    .unwrap(),
                            )
                            .unwrap();
                        Ok(Redirect::to("/.toki/mod"))
                    }
                    Err(_) => Err(LoginError::InvalidCredentials),
                }
            }
        }
    }

    pub async fn get_post(&self, id: i32) -> Post {
        sqlx::query_as!(
                Post,
                r#"
                 SELECT id, parent, board, created, op, email, body, subject, files FROM posts where id = $1
            "#,
                id,
            )
            .fetch_one(&self.pool)
            .await
            .expect("Oops")
    }

    pub async fn children(&self, parent: i32) -> Option<Vec<Post>> {
        let children = sqlx::query_as!(
                Post,
                r#"
                 SELECT id, parent, board, created, op, email, body, subject, files FROM posts where parent = $1
            "#,
                parent,
            )
            .fetch_all(&self.pool)
            .await
            .expect("Oops");

        match children.len() {
            0 => None,
            _ => Some(children),
        }
    }

    pub async fn create_post(&self, input: &Input) {
        dbg!("creating post...");
        sqlx::query!(
            r#"
                     INSERT INTO posts(board, parent, op, email, body, subject, files)
                     VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            input.board,
            input.parent,
            input.op,
            input.email,
            input.body,
            input.subject,
            input.files.as_deref(),
        )
        .execute(&self.pool)
        .await
        .expect("Oops");
    }

    pub async fn get_boards(&self) -> Vec<Board> {
        sqlx::query_as!(
            Board,
            r#"
                 SELECT name, title FROM boards
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .expect("Oops")
    }

    // TODO: make `save_files` run asynchronously
    pub async fn parse_fields(&self, mut multipart: Multipart) -> Result<Input> {
        let mut result: Input = Default::default();

        while let Some(field) = multipart.next_field().await? {
            let name = field.name().unwrap().to_string();
            let data: &[u8] = &field.bytes().await?;

            match data.len() {
                0 => {}
                _ => match data.sniff_mime_type() {
                    Some(x) => {
                        dbg!(&x);
                        let extension = x.split('/').nth(1).unwrap().to_string();
                        let hash = self.save_file(name, extension, data).await?;
                        match result.files.as_mut() {
                            Some(f) => f.push(hash),
                            None => result.files = Some(vec![hash]),
                        }
                    }
                    None => match name.as_str() {
                        "op" => result.op = std::str::from_utf8(data).unwrap().to_owned(),
                        "subject" => result.subject = std::str::from_utf8(data).unwrap().to_owned(),
                        "email" => result.email = std::str::from_utf8(data).unwrap().to_owned(),
                        "body" => result.body = std::str::from_utf8(data).unwrap().to_owned(),
                        "captcha" => result.captcha = std::str::from_utf8(data).unwrap().to_owned(),
                        "parent" => {
                            result.parent = Some((data[0] as char).to_digit(10).unwrap() as i32)
                        }
                        "board" => result.board = std::str::from_utf8(data).unwrap().to_owned(),
                        x if x.starts_with("file") => {
                            return Err(InputError::Files(
                                x.chars().last().unwrap().to_digit(10).unwrap(),
                            )
                            .into())
                        }
                        _ => panic!(),
                    },
                },
            }
        }

        dbg!(&result);
        event!(Level::INFO, "result: {:?}", result);
        Ok(result)
    }

    pub async fn save_file(&self, name: String, extension: String, data: &[u8]) -> Result<String> {
        let mut hasher = Ripemd160::new();

        hasher.update(name);
        let hash = hasher.finalize();
        let hash_ext = format!("{:x}.{}", hash, extension);
        dbg!("file saved as {}", &hash_ext);
        let mut file = File::create(&hash_ext).await?;

        file.write_all(data).await?;
        file.flush().await?;

        Ok(hash_ext)
    }
}
