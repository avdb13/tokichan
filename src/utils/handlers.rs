use anyhow::Result;
use axum::http::Method;
use axum::http::StatusCode;
use axum::response::Response;
use axum::BoxError;
use axum::Extension;
use axum_sessions::extractors::ReadableSession;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tower::timeout::error::Elapsed;

use super::data::Credentials;
use super::templates::*;
use crate::App;
use axum::debug_handler;
use axum::extract::Multipart;
use axum::extract::Path;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::Redirect;
use axum::Form;
use axum::{http::Uri, response::Html};
use axum_sessions::extractors::WritableSession;

pub async fn get_root(
    State(app): State<Arc<App>>,
    Extension(session): Extension<bool>,
) -> impl IntoResponse {
    HtmlTemplate(HomeTemplate {
        base: BaseTemplate {
            authenticated: session,
            current_year: 2022u32,
            boards: app.boards.clone(),
            captcha: Some("foobar".to_owned()),
            flash: None,
        },
    })
}

pub async fn get_recent(
    State(app): State<Arc<App>>,
    Extension(session): Extension<bool>,
) -> Response {
    sleep(Duration::from_secs(3)).await;
    let posts = app.models.recent().await;

    HtmlTemplate(BoardTemplate {
        base: BaseTemplate {
            authenticated: session,
            current_year: 2022u32,
            boards: app.boards.clone(),
            captcha: Some("foobar".to_owned()),
            flash: None,
        },
        board: "recent".to_owned(),
        posts,
        input: Input::default(),
    })
    .into_response()
}

pub async fn get_mod(State(app): State<Arc<App>>, Extension(session): Extension<bool>) -> Response {
    if session {
        HtmlTemplate(ModTemplate {
            credentials: Credentials {
                username: "".to_owned(),
                password: "".to_owned(),
                role: None,
            },
            base: BaseTemplate {
                authenticated: session,
                current_year: 2022u32,
                boards: app.boards.clone(),
                captcha: Some("foobar".to_owned()),
                flash: None,
            },
        })
        .into_response()
    } else {
        Redirect::to("/").into_response()
    }
}

pub async fn get_login(
    State(app): State<Arc<App>>,
    // Extension(session): Extension<bool>,
) -> impl IntoResponse {
    dbg!("hello");
    // if session {
    //     Redirect::to("/.toki/mod").into_response()
    // } else {
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
    .into_response()
    // }
}

pub async fn get_signup(
    State(app): State<Arc<App>>,
    // Extension(session): Extension<bool>,
) -> impl IntoResponse {
    HtmlTemplate(SignupTemplate {
        credentials: Credentials {
            username: "".to_owned(),
            password: "".to_owned(),
            role: None,
        },
        base: BaseTemplate {
            authenticated: true,
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
    //     authenticated: session,
    //     input: Input::default(),
    // })
}

pub async fn get_board(
    State(app): State<Arc<App>>,
    Path(board): Path<String>,
    Extension(session): Extension<bool>,
) -> impl IntoResponse {
    if !app.boards.iter().any(|x| x.name == board) {
        return not_found(Path(board)).await.into_response();
    }
    let posts = app.models.get_board(board.clone()).await;

    HtmlTemplate(BoardTemplate {
        base: BaseTemplate {
            authenticated: session,
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
    State(app): State<Arc<App>>,
    Path((board, id)): Path<(String, String)>,
    Extension(session): Extension<bool>,
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
            authenticated: session,
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

pub async fn signup(
    State(app): State<Arc<App>>,
    // Extension(session): Extension<bool>,
    Form(credentials): Form<Credentials>,
) -> Redirect {
    dbg!(&credentials);
    let result = app.models.signup(credentials).await;
    Redirect::to("/.toki/mod")
}

#[debug_handler]
pub async fn login(
    State(app): State<Arc<App>>,
    Extension(_session): Extension<bool>,
    mut session: WritableSession,
    Form(credentials): Form<Credentials>,
) -> Response {
    match app.models.login(credentials).await {
        Ok(x) => {
            session.insert("signed_in", true).unwrap();
            Redirect::to("/.toki/mod").into_response()
        }
        Err(e) => HtmlTemplate(LoginTemplate {
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
                flash: Some(e.to_string()),
            },
        })
        .into_response(),
    }
}

pub async fn logout(
    State(app): State<Arc<App>>,
    mut session: WritableSession,
) -> impl IntoResponse {
    session.destroy();
    Redirect::to("/")
}

pub async fn create_post(
    State(app): State<Arc<App>>,
    Extension(session): Extension<bool>,
    multipart: Multipart,
) -> Redirect {
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

pub async fn captcha() -> Html<String> {
    unimplemented!()
}

pub async fn fallback(path: Uri) -> impl IntoResponse {
    format!("Oops! No {}", path)
}

pub async fn timeout(method: Method, uri: Uri, error: BoxError) -> impl IntoResponse {
    match error {
        x if x.is::<Elapsed>() => (
            StatusCode::REQUEST_TIMEOUT,
            format!("request timeout: {method} on {uri}"),
        ),
        x => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("internal server error: {method} on {uri}"),
        ),
    }
}
