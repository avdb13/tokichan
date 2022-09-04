use crate::data::Post;
use chrono::Utc;
use fake::{
    faker::{internet::raw::FreeEmail, lorem::raw::Sentence, lorem::raw::Words, name::raw::Name},
    locales::EN,
    Fake,
};
use sqlx::postgres::PgPool;

pub async fn populate_db(pool: PgPool) {
    for i in 0..100 {
        let post = Post {
            id: i as i32,
            parent: None,
            board: "b".to_string(),
            created: Utc::now(),
            children: None,

            op: Name(EN).fake(),
            email: Some(FreeEmail(EN).fake()),
            body: Some(Sentence(EN, 1..5).fake()),
            subject: Some(Words(EN, 1..5).fake::<Vec<String>>().join(" ")),
            files: None,
        };

        sqlx::query!(
            "
           INSERT INTO posts (id, board, parent, op, email, body, subject)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
        ",
            post.id,
            post.board,
            post.parent,
            post.op,
            post.email,
            post.subject,
            post.body
        )
        .execute(&pool)
        .await
        .expect("Oops");
    }
}
