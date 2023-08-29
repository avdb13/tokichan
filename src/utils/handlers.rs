use axum::headers::Header;

use axum::http::Method;
use axum::http::StatusCode;
use axum::response::Response;

use axum::BoxError;
use axum::Extension;
use axum_sessions::extractors::ReadableSession;

use color_eyre::Result;
use digest::Digest;
use hmac::Mac;
use http_body::Full;
use tokio::time::sleep;

use std::sync::Arc;
use std::time::Duration;

use tower::timeout::error::Elapsed;

use super::data::Credentials;

use super::error::RequestError;
use super::templates::*;
use crate::App;
use axum::debug_handler;

use axum::extract::Path;
use axum::extract::State;
use axum::http::Uri;
use axum::response::IntoResponse;
use axum::response::Redirect;
use axum::Form;
use axum_sessions::extractors::WritableSession;

pub async fn get_root(State(app): State<Arc<App>>, session: ReadableSession) -> impl IntoResponse {
    HtmlTemplate(HomeTemplate {
        base: BaseTemplate {
            authenticated: session.get("authenticated").unwrap_or(false),
            current_year: 2022u32,
            boards: app.boards.clone(),
            captcha: Some("foobar".to_owned()),
            flash: None,
        },
    })
}

pub async fn get_recent(State(app): State<Arc<App>>, session: ReadableSession) -> Response {
    let posts = app.models.recent().await;

    HtmlTemplate(BoardTemplate {
        base: BaseTemplate {
            authenticated: session.get("authenticated").unwrap_or(false),
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

pub async fn get_mod(State(app): State<Arc<App>>, session: ReadableSession) -> Response {
    if let Some(_) = session.get::<bool>("authenticated") {
        HtmlTemplate(ModTemplate {
            credentials: Credentials {
                username: "".to_owned(),
                ..Default::default()
            },
            base: BaseTemplate {
                authenticated: true,
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

pub async fn get_login(State(app): State<Arc<App>>, session: ReadableSession) -> impl IntoResponse {
    if let Some(_) = session.get::<bool>("authenticated") {
        Redirect::to("/.toki/mod").into_response()
    } else {
        HtmlTemplate(LoginTemplate {
            credentials: Credentials {
                username: "".to_owned(),
                ..Default::default()
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
    }
}

pub async fn get_signup(
    State(app): State<Arc<App>>,
    _session: ReadableSession,
) -> impl IntoResponse {
    HtmlTemplate(SignupTemplate {
        credentials: Credentials {
            username: "".to_owned(),
            ..Default::default()
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

// TODO: fix path
pub async fn get_board(
    State(app): State<Arc<App>>,
    Path(board): Path<String>,
    session: ReadableSession,
) -> impl IntoResponse {
    if !app.boards.iter().any(|x| x.name == board) {
        dbg!("not found!");
        return not_found(Path(board)).await.into_response();
    }
    let posts = app.models.get_board(board.clone()).await;

    HtmlTemplate(BoardTemplate {
        base: BaseTemplate {
            authenticated: session.get("authenticated").unwrap_or(false),
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
    session: WritableSession,
) -> impl IntoResponse {
    // sleep(Duration::from_secs(8)).await;
    if id.parse::<u32>().is_err() || id.parse::<i32>().is_err() {
        return not_found(Path(id)).await.into_response();
    }

    let id = id.parse::<i32>().unwrap();

    let post = app.models.get_post(id).await;
    // let captcha = generate();

    if let Some(p) = post.parent {
        return not_found(Path(p.to_string())).await.into_response();
    }

    let children = app.models.children(id).await;

    HtmlTemplate(ThreadTemplate {
        invalid_captcha: false,
        base: BaseTemplate {
            authenticated: true,
            // authenticated: session,
            current_year: 2022u32,
            boards: app.boards.clone(),
            captcha: Some("".to_owned()),
            flash: session.get("flash"),
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
    // Extension(session): Extension<Session>,
    Form(credentials): Form<Credentials>,
) -> Redirect {
    dbg!(&credentials);
    let _result = app.models.signup(credentials).await;
    Redirect::to("/.toki/mod")
}

#[debug_handler]
pub async fn login(
    State(app): State<Arc<App>>,
    mut session: WritableSession,
    Form(credentials): Form<Credentials>,
) -> Response {
    match app.models.login(credentials).await {
        Ok(_x) => {
            session.insert("signed_in", true).unwrap();
            Redirect::to("/.toki/mod").into_response()
        }
        Err(e) => HtmlTemplate(LoginTemplate {
            credentials: Credentials {
                username: "".to_owned(),
                ..Default::default()
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
    State(_app): State<Arc<App>>,
    mut session: WritableSession,
) -> impl IntoResponse {
    session.destroy();
    Redirect::to("/")
}

pub async fn captcha(Extension(bytes): Extension<Vec<u8>>) -> Response {
    Response::builder()
        .header("Content-Type", "image/png")
        .body(Full::from(bytes))
        .unwrap()
        .into_response()
}

pub async fn create_post(
    State(app): State<Arc<App>>,
    Extension(input): Extension<Result<Input, RequestError>>,
) -> Response {
    match input {
        Ok(input) => match app.models.create_post(&input).await {
            Ok(_) => match input.parent {
                Some(p) => Redirect::to(format!("/{}/{}", input.board, p).as_str()).into_response(),
                None => Redirect::to(format!("/{}", input.board).as_str()).into_response(),
            },
            Err(e) => e.to_string().into_response(),
        },
        Err(e) => e.to_string().into_response(),
    }
}

pub async fn fallback(path: Uri) -> impl IntoResponse {
    format!("Oops! No {}", path)
}

pub async fn timeout(method: Method, uri: Uri, err: BoxError) -> impl IntoResponse {
    if err.is::<Elapsed>() {
        (
            StatusCode::REQUEST_TIMEOUT,
            format!("request timeout: {method} on {uri}"),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("internal server error: {method} on {uri}"),
        )
    }
}
