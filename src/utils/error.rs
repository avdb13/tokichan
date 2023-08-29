use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use color_eyre::Report;
use thiserror::Error;

use super::{
    data::Credentials,
    templates::{BaseTemplate, HtmlTemplate, LoginTemplate},
};

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

#[derive(Clone, Error, Debug)]
pub enum RequestError {
    #[error("incorrect captcha")]
    IncorrectCaptcha,
    #[error("body exceeded allowed limit")]
    SizeLimit,
    #[error("request timeout")]
    TimeoutLimit,
    #[error("no key for one or more fields")]
    MissingKey,
}

impl IntoResponse for LoginError {
    fn into_response(self) -> Response {
        HtmlTemplate(LoginTemplate {
            credentials: Credentials {
                username: "".to_owned(),
                ..Default::default()
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

pub struct AppError(Report);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("error: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<Report>,
{
    fn from(e: E) -> Self {
        Self(e.into())
    }
}
