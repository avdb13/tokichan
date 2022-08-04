use askama::Template;
use serde::Deserialize;
use axum::response::{
    Html,
    Response,
    IntoResponse,
};
use http::StatusCode;

#[derive(Template)]
#[template(path = "home.page.html")]
pub struct HomeTemplate {
// AuthenticatedUser: data.User,
	// CSRFToken:         String,
	pub current_year:       u32,
	pub boards:             Vec<String>,
	pub captcha:            String,
    pub flash:              bool,
    pub authenticated_user: bool,

	// Form *forms.Form
	// Post *data.Post

	// Posts  *[]data.Post
	// Boards *[]data.Board
}

#[derive(Template)]
#[template(path = "create.page.html")]
pub struct PostTemplate {
// AuthenticatedUser: data.User,
	// CSRFToken:         String,
	pub current_year:       u32,
	pub board:              String,
    pub boards:             Vec<String> ,
	pub captcha:            String,
    pub flash:              bool,
    pub authenticated_user: bool,
    pub input:              Input,

	// pub form: Form,
    // pub post: Post,

	// Posts  *[]data.Post
	// Boards *[]data.Board
}

#[derive(Deserialize, Default, Debug)]
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
