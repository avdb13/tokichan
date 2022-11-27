use std::sync::Arc;

use super::data::Credentials;
use super::templates::*;
use crate::App;
use axum::debug_handler;
use axum::extract::Multipart;
use axum::extract::Path;
use axum::response::Redirect;
use axum::Extension;
use axum::Form;
use axum::{http::Uri, response::Html};
use axum_core::response::IntoResponse;
use axum_sessions::extractors::ReadableSession;
use axum_sessions::extractors::WritableSession;
use chrono::DateTime;
use chrono::Utc;

pub async fn get_root(Extension(app): Extension<Arc<App>>) -> impl IntoResponse {
    HtmlTemplate(HomeTemplate {
        base: BaseTemplate {
            authenticated: false,
            current_year: 2022u32,
            boards: app.boards.clone(),
            captcha: Some("foobar".to_owned()),
            flash: None,
        },
    })
}

pub async fn get_mod(
    Extension(app): Extension<Arc<App>>,
    session: ReadableSession,
) -> impl IntoResponse {
    if session.get::<DateTime<Utc>>("mikoto").is_none() {
        dbg!("logged out!");
    };
    HtmlTemplate(ModTemplate {
        credentials: Credentials {
            username: "".to_owned(),
            password: "".to_owned(),
            role: None,
        },
        base: BaseTemplate {
            authenticated: false,
            current_year: 2022u32,
            boards: app.boards.clone(),
            captcha: Some("foobar".to_owned()),
            flash: None,
        },
    })
}

pub async fn get_login(Extension(app): Extension<Arc<App>>) -> impl IntoResponse {
    HtmlTemplate(LoginTemplate {
        credentials: Credentials {
            username: "".to_owned(),
            password: "".to_owned(),
            role: None,
        },
        base: BaseTemplate {
            authenticated: false,
            current_year: 2022u32,
            boards: app.boards.clone(),
            captcha: Some("foobar".to_owned()),
            flash: None,
        },
    })
}

pub async fn get_signup(Extension(app): Extension<Arc<App>>) -> impl IntoResponse {
    HtmlTemplate(SignupTemplate {
        credentials: Credentials {
            username: "".to_owned(),
            password: "".to_owned(),
            role: None,
        },
        base: BaseTemplate {
            authenticated: false,
            current_year: 2022u32,
            boards: app.boards.clone(),
            captcha: Some("foobar".to_owned()),
            flash: None,
        },
    })
}

pub async fn not_found(Path(board): Path<String>) -> impl IntoResponse {
    format!("You came from {board}!")
    // HtmlTemplate(BoardTemplate {
    //     posts,
    //     current_year: 2022i32,
    //     board: "b".to_owned(),
    //     boards: app.boards.clone(),
    //     captcha: "foobar".to_owned(),
    //     flash: false,
    //     authenticated: false,
    //     input: Input::default(),
    // })
}

pub async fn get_board(
    Extension(app): Extension<Arc<App>>,
    Path(board): Path<String>,
) -> impl IntoResponse {
    if !app.boards.iter().any(|x| x.name == board) {
        return not_found(Path(board)).await.into_response();
    }
    let posts = app.models.get_board(board.clone()).await;

    HtmlTemplate(BoardTemplate {
        base: BaseTemplate {
            authenticated: false,
            current_year: 2022u32,
            boards: app.boards.clone(),
            captcha: Some("foobar".to_owned()),
            flash: None,
        },
        board,
        posts,
        input: Input::default(),
    })
    .into_response()
}

pub async fn get_post(
    Extension(app): Extension<Arc<App>>,
    Path((board, id)): Path<(String, String)>,
) -> impl IntoResponse {
    if id.parse::<u32>().is_err() || id.parse::<i32>().is_err() {
        return not_found(Path(id)).await.into_response();
    }

    let id = id.parse::<i32>().unwrap();

    let post = app.models.get_post(id).await;

    if let Some(p) = post.parent {
        return not_found(Path(p.to_string())).await.into_response();
    }

    let children = app.models.children(id).await;

    HtmlTemplate(ThreadTemplate {
        base: BaseTemplate {
            authenticated: false,
            current_year: 2022u32,
            boards: app.boards.clone(),
            captcha: Some("foobar".to_owned()),
            flash: None,
        },
        board,
        post,
        children,
        input: Input::default(),
    })
    .into_response()
}

#[debug_handler]
pub async fn signup(
    Extension(app): Extension<Arc<App>>,
    Form(credentials): Form<Credentials>,
) -> Redirect {
    dbg!(&credentials);
    let result = app.models.signup(credentials).await;
    Redirect::to("/.toki/mod")
}

#[debug_handler]
pub async fn login(
    Extension(app): Extension<Arc<App>>,
    Form(credentials): Form<Credentials>,
    session: WritableSession,
) -> impl IntoResponse {
    app.models.login(credentials, session).await
}

#[debug_handler]
pub async fn logout(
    Extension(app): Extension<Arc<App>>,
    // Form(credentials): Form<Credentials>,
    session: WritableSession,
) -> impl IntoResponse {
    app.models.logout("mikoto".to_owned(), session).await
}

#[debug_handler]
pub async fn create_post(Extension(app): Extension<Arc<App>>, multipart: Multipart) -> Redirect {
    // let input: Input = Default::default();
    // app.models.create_post(&input).await;
    let result = app.models.parse_fields(multipart).await;

    match result {
        Ok(input) => {
            app.models.create_post(&input).await;

            match input.parent {
                Some(p) => {
                    dbg!(&format!("/{}/{}", input.board, p).as_str());
                    Redirect::to(format!("/{}/{}", input.board, p).as_str())
                }
                None => {
                    dbg!(&format!("/{}", input.board).as_str());
                    Redirect::to(format!("/{}", input.board).as_str())
                }
            }
        }
        Err(e) => {
            dbg!("oops: {:?}", e);
            Redirect::to("")
        }
    }
}

pub async fn recent() -> Html<String> {
    unimplemented!()
}

pub async fn captcha() -> Html<String> {
    unimplemented!()
}

pub async fn fallback(path: Uri) -> impl IntoResponse {
    format!("Oops! No {}", path)
}
