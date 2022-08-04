use axum::{
    response::Html,
    extract::Form,
    http::{Uri, StatusCode},
};
use crate::templates::*;
use axum_macros::debug_handler;
use axum_core::response::IntoResponse;

#[debug_handler]
pub async fn get_root() -> impl IntoResponse {
    HtmlTemplate(HomeTemplate {
        current_year: 2022u32,
        boards: vec!["b".to_owned(), "l".to_owned(), "g".to_owned()],
        captcha: "foobar".to_owned(),
        flash: false,
        authenticated_user: false,
    })
}

pub async fn get_board() -> Html<String> {
    unimplemented!()
}

pub async fn get_post() -> impl IntoResponse {
    HtmlTemplate(PostTemplate {
        current_year: 2022u32,
        board: "b".to_owned(),
        boards: vec!["b".to_owned(), "l".to_owned(), "g".to_owned()],
        captcha: "foobar".to_owned(),
        flash: false,
        authenticated_user: false,
        input: Input::default(),
    })
}

pub async fn create_post(Form(input): Form<Input>) {
    dbg!(&input);
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
