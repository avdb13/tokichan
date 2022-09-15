use crate::data::Post;
use chrono::Utc;
use fake::{
    faker::{internet::raw::FreeEmail, lorem::raw::Sentence, lorem::raw::Words, name::raw::Name},
    locales::EN,
    Fake,
};
use rand::seq::SliceRandom;
use sqlx::postgres::PgPool;
use tokio::fs::{read_dir, DirEntry};

pub async fn populate_db(pool: PgPool) {
    let mut files = read_dir("/home/mikoto/Downloads").await.unwrap();
    let mut examples: Vec<String> = Vec::new();

    while let Some(f) = files.next_entry().await.unwrap() {
        if [".jpeg", ".png", ".jpg"]
            .iter()
            .any(|s| f.path().to_str().unwrap().ends_with(s))
        {
            let name = f.file_name().into_string().unwrap();
            std::fs::copy(
                f.path().to_str().unwrap(),
                format!("./ui/static/images/{}", name).as_str(),
            )
            .unwrap();
            examples.push(name);
        }
    }

    for i in 0..100 {
        let files = (0..3)
            .map(|_| {
                examples
                    .choose(&mut rand::thread_rng())
                    .unwrap()
                    .to_string()
            })
            .collect::<Vec<String>>();

        dbg!(&files);

        let post = Post {
            id: i as i32,
            parent: None,
            board: "b".to_string(),
            created: Utc::now(),

            op: Name(EN).fake(),
            email: Some(FreeEmail(EN).fake()),
            body: Some(Sentence(EN, 1..5).fake()),
            subject: Some(Words(EN, 1..5).fake::<Vec<String>>().join(" ")),
            files: Some(files),
        };

        sqlx::query!(
            "
           INSERT INTO posts (board, parent, op, email, body, subject, files)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
        ",
            post.board,
            post.parent,
            post.op,
            post.email,
            post.subject,
            post.body,
            &post.files.unwrap()[..],
        )
        .execute(&pool)
        .await
        .expect("Oops");
    }
}
