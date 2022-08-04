use chrono::Utc;
use crate::data::{Post, Fields};
use sqlx::postgres::PgConnection;
use fake::{
    Fake,
    faker::{
        lorem::raw::Paragraph,
        name::raw::Name,
        lorem::raw::Sentence,
        internet::raw::FreeEmail,
    },
    locales::EN,
};

pub async fn populate_db(client: &PgConnection) {
    for i in 0..100 {
        let post = Post {
            id: Default::default(),
            parent: Default::default(),
            board: "b".to_string(),
            created: Utc::now(),
            children: vec![],

            fields: Fields {
                op: Name(EN).fake(),
                email: FreeEmail(EN).fake(),
                body: Sentence(EN, 0..3).fake(),
                subject: Paragraph(EN, 0..3).fake(),
                files: vec![],
            },
        };
       sqlx::query!(r#"
           INSERT INTO posts (op, email, subject, body) VALUES ($1, $2, $3, $4)
        "#,
        post.fields.op,
        post.fields.email,
        post.fields.subject,
        post.fields.body
       ).execute().await.expect("Oops");
    }
}
