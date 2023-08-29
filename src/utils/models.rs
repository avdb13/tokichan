use std::io::Error;

use crate::utils::error::RequestError;
use crate::utils::helpers::{format_name, save_file};

use super::data::*;
use super::error::LoginError;
use super::templates::Input;
use axum::response::Redirect;
use axum_sessions::extractors::WritableSession;
use bevy_reflect::GetField;

use color_eyre::{Report, Result};

use mime_sniffer::MimeTypeSniffer;
use password_hash::rand_core::OsRng;
use password_hash::SaltString;
use pbkdf2::password_hash::PasswordVerifier;
use pbkdf2::Pbkdf2;

use ripemd::Digest;
use sqlx::PgPool;

use thiserror::Error;
use tokio::task;

use tracing::info;

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

    pub async fn get_board(&self, board: String) -> Vec<Post> {
        sqlx::query_as!(
            Post,
            r#"
             SELECT id, parent, board, created, op, email, body, subject, files FROM posts
             WHERE parent IS NULL AND board = $1

             ORDER BY created DESC LIMIT 100
         "#,
            board,
        )
        .fetch_all(&self.pool)
        .await
        .expect("Oops")
    }

    pub async fn recent(&self) -> Vec<Post> {
        sqlx::query_as!(
            Post,
            r#"
             SELECT id, parent, board, created, op, email, body, subject, files FROM posts
             WHERE parent IS NULL
             ORDER BY created DESC LIMIT 100
        "#,
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

        let salt = SaltString::generate(&mut OsRng).to_string();

        sqlx::query!(
            r#"
                     INSERT INTO users(name, password, salt, role)
                     VALUES ($1, $2, $3, $4)
                "#,
            credentials.username,
            credentials.hash().to_string(),
            salt,
            format!("{:?}", credentials.role).to_lowercase(),
        )
        .execute(&self.pool)
        .await
        .expect("Oops");
        Ok(())
    }

    pub async fn logout(&self, username: String, mut session: WritableSession) -> Redirect {
        session.remove(&username);
        info!("{} logged out!", username);
        // Inject flash message
        Redirect::to("/")
    }

    pub async fn login(&self, credentials: Credentials) -> Result<i32, LoginError> {
        if credentials.username.is_empty() {
            return Err(LoginError::EmptyUsername);
        } else if credentials.password.is_empty() {
            return Err(LoginError::EmptyPassword);
        }

        let result = sqlx::query!(
            r#"
             SELECT id, password, salt FROM users where name = $1
        "#,
            credentials.username,
        )
        .fetch_optional(&self.pool)
        .await
        .expect("Oops");

        match result {
            None => Err(LoginError::NonExistentUser(credentials.username)),
            Some(record) => {
                let expected_credentials = Credentials {
                    password: record.password,
                    salt: record.salt,
                    ..Default::default()
                };

                match Pbkdf2.verify_password(
                    credentials.hash().to_string().as_bytes(),
                    &expected_credentials.hash(),
                ) {
                    Ok(_) => Ok(record.id),
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

    pub async fn create_post(&self, input: &Input) -> Result<(), Error> {
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

        sqlx::query!(
            r#"
                UPDATE boards SET posts = posts + 1 WHERE name = $1
                "#,
            input.board
        )
        .execute(&self.pool)
        .await
        .expect("Oops");

        Ok(())
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

    pub async fn parse_fields(
        &self,
        mut multipart: multer::Multipart<'_>,
    ) -> Result<Input, Report> {
        let mut result: Input = Default::default();
        let mut files: Vec<_> = Vec::new();

        while let Some(field) = multipart.next_field().await? {
            let key = field.name().ok_or(RequestError::MissingKey)?.to_owned();
            let value: Vec<u8> = field.bytes().await?.to_vec();

            match value.len() {
                0 => {}
                _ => match value.clone().sniff_mime_type() {
                    Some(mime) => {
                        let id = &value[0..32];
                        let name = format_name((id, mime.split('/').last().unwrap().to_owned()));
                        files.push(name.clone());

                        task::spawn(async move { save_file(name, &value).await });
                    }
                    None => {
                        // edge-case since bevy_reflect forces the user to downcast T
                        match key.as_str() {
                            "parent" => {
                                // 49 == ASCII for 1, stupid because if value[0]::<u8> >= u8::MAX
                                // this will result in undefined behavior
                                result.parent = Some((value[0] - 48) as i32)
                            }
                            _ => {
                                *result.get_field_mut::<String>(&key).unwrap() =
                                    std::str::from_utf8(&value).unwrap().to_owned();
                            }
                        }
                    }
                },
            }
        }

        info!("created post: {:?}", result);

        result.files = Some(files);
        Ok(result)
    }

    // pub async fn merge_thread(&self, old: u32, new: u32) -> Result<()> {
    //     let posts = sqlx::query!(
    //         r#"
    //             UPDATE posts SET parent = $1 WHERE parent = $2 OR id = $2
    //             "#,
    //     )
    //     .fetch_all(&self.pool)
    //     .await
    //     .expect("Oops");
    //     Ok(())
    // }
}
