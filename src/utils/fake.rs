use std::{fs, io::ErrorKind};

use super::{
    data::Post,
    helpers::{format_name, save_file},
};
use chrono::Utc;
use color_eyre::Report;
use fake::{
    faker::{internet::raw::FreeEmail, lorem::raw::Sentence, lorem::raw::Words, name::raw::Name},
    locales::EN,
    Fake,
};
use futures::future::try_join_all;
use rand::seq::SliceRandom;
use roux::{util::FeedOption, Subreddit};
use sqlx::postgres::PgPool;

use tracing::info;

pub struct ImagePool<'a> {
    pool: &'a PgPool,
    images: Vec<String>,
}

impl<'a> ImagePool<'a> {
    pub async fn new(pool: &'a PgPool, size: usize) -> Result<ImagePool, Report> {
        let fetch = |s: String| async move {
            let http_client = reqwest::Client::new();

            let resp = http_client.get(s.clone()).send().await?;
            let bytes = resp.bytes().await?;

            let name: Vec<_> = s.split('/').collect();
            let name = name.last().unwrap();
            let name: Vec<_> = name.split('.').collect();

            let name = format_name((name[0].as_bytes(), name[1].to_owned()));
            save_file(name.clone(), &bytes).await?;

            Ok::<String, Report>(name.to_owned())
        };

        let r = match fs::read_dir("./.tmp") {
            Ok(mut r) => {
                if r.next().is_some() {
                    Some(r)
                } else {
                    None
                }
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {
                fs::create_dir("./.tmp")?;
                None
            }
            Err(e) => return Err(e.into()),
        };
        match r {
            Some(r) => {
                info!("mocker images already present");
                let images = r
                    .into_iter()
                    .map(|s| s.unwrap().file_name().to_str().unwrap().to_owned())
                    .collect();

                Ok(Self { pool, images })
            }
            None => {
                info!("fetching mocker images");
                let images = Subreddit::new("onetruebiribiri")
                    .top(
                        100,
                        Some(FeedOption::new().period(roux::util::TimePeriod::ThisYear)),
                    )
                    .await?;

                // filter out videos, text-only threads
                let files: _ = images
                    .data
                    .children
                    .iter()
                    .flat_map(|x| x.data.url.clone())
                    .filter(|x| x.contains('.'))
                    .take(size)
                    .map(fetch);

                let images = try_join_all(files).await?;
                Ok(Self { pool, images })
            }
        }
    }

    pub async fn truncate(&mut self) -> Result<(), Report> {
        let old_files = sqlx::query!(
            "
            SELECT files FROM posts
        ",
        )
        .fetch_all(self.pool)
        .await?;

        old_files
            .iter()
            .flat_map(|x| x.files.to_owned())
            .flatten()
            .try_for_each(fs::remove_file)
            .map_err(|e| info!("failed to delete image: {}", e))
            .ok();
        info!("deleting old mocker images");

        sqlx::query!(
            "
            TRUNCATE posts
        ",
        )
        .execute(self.pool)
        .await?;

        sqlx::query!(
            "
            ALTER SEQUENCE posts_id_seq RESTART
        ",
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn mock(&mut self, ops: usize, children: usize) -> Result<(), Report> {
        info!("mocking ...");

        for _ in 1..=ops {
            let files = self.random_files(1).await;
            let post = self.create(files, None).await;
            self.insert(post).await?;
        }
        for i in 1..=ops {
            let res = (0..children)
                .map(|_| async {
                    let files = self.random_files(1).await;
                    let child = self.create(files, Some(i as i32)).await;
                    self.insert(child).await
                })
                .collect::<Vec<_>>();

            try_join_all(res).await?;
        }

        Ok(())
    }

    async fn create(&self, files: Vec<String>, parent: Option<i32>) -> Post {
        Post {
            id: 0,
            parent,
            board: "b".to_string(),
            created: Utc::now(),

            op: Name(EN).fake(),
            email: Some(FreeEmail(EN).fake()),
            body: Some(Sentence(EN, 1..5).fake()),
            subject: Some(Words(EN, 1..5).fake::<Vec<String>>().join(" ")),
            files: Some(files),
        }
    }

    async fn insert(&self, post: Post) -> Result<(), Report> {
        let files = if let Some(files) = post.files {
            files
        } else {
            Vec::new()
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
            &files,
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn random_files(&self, amount: usize) -> Vec<String> {
        (0..amount)
            .map(|_| {
                self.images
                    .choose(&mut rand::thread_rng())
                    .unwrap()
                    .to_owned()
            })
            .collect()
    }
}

#[sqlx::test]
async fn test_new(pool: PgPool) -> Result<(), Report> {
    let object = ImagePool::new(&pool, 3).await?;
    let dir = fs::read_dir("./.tmp/")?;
    let len = dir.into_iter().count();

    assert_eq!(len, object.images.len());

    Ok(())
}

#[sqlx::test]
async fn test_truncate(pool: PgPool) -> Result<(), Report> {
    let mut object = ImagePool::new(&pool, 3).await?;
    object.truncate().await?;

    let v = sqlx::query!(
        "
        SELECT id FROM posts
    ",
    )
    .fetch_all(&pool)
    .await?;

    assert_eq!(v.len(), 0);

    Ok(())
}

#[sqlx::test]
async fn test_mock(pool: PgPool) -> Result<(), Report> {
    let mut object = ImagePool::new(&pool, 3).await?;
    object.mock(5, 3).await?;

    let parents = sqlx::query!(
        "
        SELECT id FROM posts WHERE PARENT IS NULL
    ",
    )
    .fetch_all(&pool)
    .await?;

    for p in parents {
        let children = sqlx::query!(
            "
            SELECT id FROM posts WHERE PARENT = $1
        ",
            p.id,
        )
        .fetch_all(&pool)
        .await?;

        assert_eq!(children.len(), 2);
    }

    Ok(())
}
