use std::sync::Arc;

use crate::templates::*;
use crate::App;
use axum::extract::Multipart;
use axum::extract::Path;
use axum::response::Redirect;
use axum::Extension;
use axum::{
    http::{StatusCode, Uri},
    response::Html,
};
use axum_core::response::IntoResponse;

pub async fn get_root(Extension(app): Extension<Arc<App>>) -> impl IntoResponse {
    HtmlTemplate(HomeTemplate {
        current_year: 2022i32,
        boards: app.boards.clone(),
        captcha: "foobar".to_owned(),
        flash: false,
        authenticated: false,
    })
}

pub async fn get_board(
    Extension(app): Extension<Arc<App>>,
    Path(board): Path<String>,
) -> impl IntoResponse {
    let posts = app.models.get_board(board).await;

    HtmlTemplate(BoardTemplate {
        posts,
        current_year: 2022i32,
        board: "b".to_owned(),
        boards: app.boards.clone(),
        captcha: "foobar".to_owned(),
        flash: false,
        authenticated: false,
        input: Input::default(),
    })
}

pub async fn get_post(
    Extension(app): Extension<Arc<App>>,
    Path((board, id)): Path<(String, String)>,
) -> impl IntoResponse {
    let id = id.parse().expect("Oops");
    let post = app.models.get_post(id).await;
    let children = app.models.children(id).await;

    HtmlTemplate(ThreadTemplate {
        id,
        current_year: 2022i32,
        boards: app.boards.clone(),
        board,
        post,
        children,
        captcha: "foobar".to_owned(),
        flash: false,
        authenticated: false,
        input: Input::default(),
    })
}

pub async fn create_post(Extension(app): Extension<Arc<App>>, multipart: Multipart) -> Redirect {
    let input: Input = Default::default();
    // app.models.create_post(&input).await;
    // app.models.save_files(multipart, input).await;

    // match input.parent {
    //     Some(p) => {
    //         dbg!(&format!("/{}/{}", input.board, p).as_str());
    //         Redirect::to(format!("/{}/{}", input.board, p).as_str())
    //     }
    //     None => {
    //         dbg!(&format!("/{}", input.board).as_str());
    //         Redirect::to(format!("/{}", input.board).as_str())
    //     }
    // }
    Redirect::to("/")
}

pub async fn recent() -> Html<String> {
    unimplemented!()
}

pub async fn captcha() -> Html<String> {
    unimplemented!()
}

pub async fn fallback(path: Uri) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("Oops! No {}", path))
}
