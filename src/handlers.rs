use std::sync::Arc;

use crate::templates::*;
use crate::App;
use axum::extract::Path;
use axum::Extension;
use axum::{
    extract::Form,
    http::{StatusCode, Uri},
    response::Html,
};
use axum_core::response::IntoResponse;

pub async fn get_root() -> impl IntoResponse {
    HtmlTemplate(HomeTemplate {
        current_year: 2022i32,
        boards: vec!["b".to_owned(), "l".to_owned(), "g".to_owned()],
        captcha: "foobar".to_owned(),
        flash: false,
        authenticated: false,
    })
}

pub async fn get_board(
    Extension(app): Extension<Arc<App>>,
    Path(board): Path<String>,
) -> impl IntoResponse {
    let posts = app.models.board(board).await;

    HtmlTemplate(BoardTemplate {
        posts,
        current_year: 2022i32,
        board: "b".to_owned(),
        boards: vec!["b".to_owned(), "l".to_owned(), "g".to_owned()],
        captcha: "foobar".to_owned(),
        flash: false,
        authenticated: false,
        input: Input::default(),
    })
}

pub async fn get_post(
    Extension(app): Extension<Arc<App>>,
    Path((_board, id)): Path<(String, String)>,
) -> impl IntoResponse {
    let id = id.parse().expect("Oops");
    let post = app.models.post(id).await;
    let children = app.models.children(id).await;

    HtmlTemplate(ThreadTemplate {
        current_year: 2022i32,
        parent: 0i32,
        board: "b".to_owned(),
        boards: vec!["b".to_owned(), "l".to_owned(), "g".to_owned()],
        post,
        children,
        captcha: "foobar".to_owned(),
        flash: false,
        authenticated: false,
        input: Input::default(),
    })
}

pub async fn create_post(Extension(app): Extension<Arc<App>>, Form(input): Form<Input>) {}

pub async fn recent() -> Html<String> {
    unimplemented!()
}

pub async fn captcha() -> Html<String> {
    unimplemented!()
}

pub async fn fallback(path: Uri) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("Oops! No {}", path))
}
