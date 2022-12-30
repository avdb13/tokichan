use super::data::Post;
use anyhow::{Error, Result};
use chrono::Utc;
use fake::{
    faker::{internet::raw::FreeEmail, lorem::raw::Sentence, lorem::raw::Words, name::raw::Name},
    locales::EN,
    Fake,
};
use futures::future::try_join_all;
use rand::{seq::SliceRandom, Rng};
use sqlx::postgres::PgPool;
use tokio::{
    fs::{read_dir, File},
    io::AsyncWriteExt,
};

pub async fn populate_db(pool: PgPool) -> Result<()> {
    let client = gelbooru_api::Client::public();
    let query = gelbooru_api::posts()
        .tag("misaka_mikoto")
        .tag("sfw")
        .limit(100)
        .send(&client)
        .await?;

    let files = query.posts.iter().map(|p| async move {
        let http_client = reqwest::Client::new();

        let resp = http_client.get(&p.file_url).send().await?;
        let bytes = resp.bytes().await?;

        let mut file = File::create("./.tmp/".to_owned() + &p.image).await?;
        file.write_all(&bytes).await?;
        Ok::<String, Error>(p.image.clone())
    });

    let files = try_join_all(files).await?;

    for i in 0..100 {
        let post = create_post(files.clone(), i, files.len()).await;

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
    Ok(())
}

pub async fn create_post(files: Vec<String>, i: usize, rng: usize) -> Post {
    let rand = (0..3)
        .map(|_| files.choose(&mut rand::thread_rng()).unwrap().clone())
        .collect();

    Post {
        id: i as i32,
        parent: None,
        board: "b".to_string(),
        created: Utc::now(),

        op: Name(EN).fake(),
        email: Some(FreeEmail(EN).fake()),
        body: Some(Sentence(EN, 1..5).fake()),
        subject: Some(Words(EN, 1..5).fake::<Vec<String>>().join(" ")),
        files: Some(rand),
    }
}
