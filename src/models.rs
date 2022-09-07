use crate::{data::*, templates::Input};
use axum::response::Redirect;
use sqlx::PgPool;

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
}
