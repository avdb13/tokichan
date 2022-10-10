use std::collections::HashMap;

use crate::data::{Board, Credentials, Post};
use anyhow::Error;
use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use sqlx::FromRow;
use struct_field_names_as_array::FieldNamesAsArray;
use thiserror::Error;

#[non_exhaustive]
pub enum Child {}

pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}

pub struct BaseTemplate {
    pub authenticated: bool,
    pub current_year: u32,
    pub boards: Vec<Board>,
    pub captcha: Option<String>,
    pub flash: Option<Result<String, String>>,
    // user
}

#[derive(Template, FromRow)]
#[template(path = "login.page.html")]
pub struct LoginTemplate {
    pub base: BaseTemplate,
    pub credentials: Credentials,
}

#[derive(Template, FromRow)]
#[template(path = "signup.page.html")]
pub struct SignupTemplate {
    pub base: BaseTemplate,
    pub credentials: Credentials,
}

#[derive(Template, FromRow)]
#[template(path = "home.page.html")]
pub struct HomeTemplate {
    pub base: BaseTemplate,
}

#[derive(Template, FromRow)]
#[template(path = "board.page.html")]
pub struct BoardTemplate {
    pub base: BaseTemplate,

    pub board: String,
    pub posts: Vec<Post>,
    pub input: Input,
}

#[derive(Template, FromRow)]
#[template(path = "thread.page.html")]
pub struct ThreadTemplate {
    pub base: BaseTemplate,

    pub board: String,
    pub post: Post,
    pub children: Option<Vec<Post>>,
    pub input: Input,
}

#[derive(Template, FromRow)]
#[template(path = "mod.page.html")]
pub struct ModTemplate {
    pub credentials: Credentials,
    pub base: BaseTemplate,
}

#[derive(FieldNamesAsArray, Deserialize, Default, Debug, FromRow)]
pub struct Input {
    pub board: String,
    pub op: String,
    pub email: String,
    pub subject: String,
    pub body: String,
    pub parent: Option<i32>,
    pub captcha: String,
    pub files: Option<Vec<String>>,
}

#[derive(Clone, Error, Debug)]
pub enum LoginError {
    #[error("user {0} does not exist")]
    NonExistentUser(String),
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("no username provided")]
    EmptyUsername,
    #[error("no password provided")]
    EmptyPassword,
}

impl IntoResponse for LoginError {
    fn into_response(self) -> Response {
        HtmlTemplate(LoginTemplate {
            credentials: Credentials {
                username: "".to_owned(),
                password: "".to_owned(),
                role: None,
            },
            base: BaseTemplate {
                authenticated: false,
                current_year: 2022u32,
                boards: vec![],
                captcha: Some("foobar".to_owned()),
                flash: None,
            },
        })
        .into_response()
    }
}
