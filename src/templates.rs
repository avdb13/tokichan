use crate::data::Post;
use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use sqlx::FromRow;

#[derive(Template, FromRow)]
#[template(path = "home.page.html")]
pub struct HomeTemplate {
    // AuthenticatedUser: data.User,
    // CSRFToken:         String,
    pub current_year: i32,
    pub boards: Vec<String>,
    pub captcha: String,
    pub flash: bool,
    pub authenticated: bool,
    // Form *forms.Form
    // Post *data.Post

    // Posts  *[]data.Post
    // Boards *[]data.Board
}

#[derive(Template, FromRow)]
#[template(path = "create.page.html")]
pub struct PostTemplate {
    pub current_year: i32,
    pub parent: i32,
    pub board: String,
    pub boards: Vec<String>,
    pub captcha: String,
    pub flash: bool,
    pub authenticated: bool,
    pub input: Input,
}

#[derive(Template, FromRow)]
#[template(path = "board.page.html")]
pub struct BoardTemplate {
    pub current_year: i32,
    pub board: String,
    pub posts: Vec<Post>,
    pub boards: Vec<String>,
    pub captcha: String,
    pub flash: bool,
    pub authenticated: bool,
    pub input: Input,
}

#[derive(Template, FromRow)]
#[template(path = "thread.page.html")]
pub struct ThreadTemplate {
    pub current_year: i32,
    pub parent: i32,
    pub board: String,
    pub boards: Vec<String>,
    pub post: Post,
    pub children: Option<Vec<Post>>,
    pub captcha: String,
    pub flash: bool,
    pub authenticated: bool,
    pub input: Input,
}

#[derive(Deserialize, Default, Debug, FromRow)]
pub struct Input {
    name: String,
    email: String,
    subject: String,
    body: String,
    pictures: [String; 3],
    captcha: String,
}

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
