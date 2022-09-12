use std::sync::{Arc, Mutex};

use crate::{data::*, templates::Input};
use anyhow::Result;
use axum::extract::Multipart;
use ripemd::{Digest, Ripemd160};
use sqlx::PgPool;
use tokio::{fs::File, io::AsyncWriteExt};

pub struct PoolModel {
    pub pool: PgPool,
}

impl PoolModel {
    pub async fn recent(&self) -> Vec<Post> {
        sqlx::query_as!(
            Post,
            r#"
             SELECT id, parent, board, created, op, email, body, subject, children, files FROM posts
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
             SELECT id, parent, board, created, op, email, body, subject, children, files FROM posts where board = $1
        "#,
            board,
        )
        .fetch_all(&self.pool)
        .await
        .expect("Oops")
    }

    pub async fn get_post(&self, id: i32) -> Post {
        sqlx::query_as!(
                Post,
                r#"
                 SELECT id, parent, board, created, op, email, body, subject, children, files FROM posts where id = $1
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
                 SELECT id, parent, board, created, op, email, body, subject, children, files FROM posts where parent = $1
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
    pub async fn save_files(&self, mut multipart: Multipart) -> Result<()> {
        while let Some(field) = multipart.next_field().await? {
            let mut hasher = Ripemd160::new();
            let name = field.name().unwrap().to_string();
            let data = field.bytes().await?;

            hasher.update(name);
            let result = hasher.finalize();
            let mut file = File::create(format!("{:x?}.{}", result, "png")).await?;

            file.write_all(&data).await?;
            file.flush().await;
        }

        Ok(())
    }
}
