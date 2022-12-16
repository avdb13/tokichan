use std::sync::Arc;

use super::{handlers, middleware::flash};
use crate::App;

use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Router,
};
use axum_extra::routing::SpaRouter;
use axum_sessions::{async_session::MemoryStore, extractors::ReadableSession, SessionLayer};
use tower_http::trace::TraceLayer;

// pub fn routes(app: Arc<App>) -> Router<Limited<Body>> {
pub fn routes(app: Arc<App>) -> Router {
    let store = MemoryStore::new();
    let secret = &(0..64).map(|_| rand::random::<u8>()).collect::<Vec<_>>();
    let session_layer = SessionLayer::new(store, secret).with_cookie_name("tokichan");
    let flash = middleware::from_fn(flash);

    let hidden = Router::new()
        .route("/login", get(handlers::get_login).post(handlers::login))
        .route("/signup", get(handlers::get_signup).post(handlers::signup))
        .route("/logout", get(handlers::logout))
        .route("/mod", get(handlers::get_mod))
        .route("/recent", get(handlers::recent))
        .route("/captcha", get(handlers::captcha));

    Router::new()
        .merge(SpaRouter::new("/static", "ui/static"))
        .route("/", get(handlers::get_root))
        .route("/:board/", get(handlers::get_board))
        .route("/:board/", post(handlers::create_post))
        .route("/:board/:id", get(handlers::get_post))
        .nest("/.toki", hidden)
        .route_layer(flash)
        .layer(session_layer)
        .layer(TraceLayer::new_for_http())
        // .layer(DefaultBodyLimit::disable())
        // .layer(RequestBodyLimitLayer::new(
        //     (app.config.security.validate_upload_limit()).unwrap(),
        // ))
        .with_state(app)
    // .fallback(handlers::not_found.into_service())

    // .route("/.toki/captcha", get(handlers::get_post))
}
