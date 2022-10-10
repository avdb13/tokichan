use std::sync::Arc;

use crate::{handlers, sessions::authenticate, App};
use axum::{
    body::Body,
    debug_handler,
    // extract::DefaultBodyLimit,
    handler::Handler,
    middleware,
    routing::{get, post},
    Extension,
    Router,
};
use axum_core::extract::DefaultBodyLimit;
use axum_extra::routing::SpaRouter;
use axum_sessions::{async_session::MemoryStore, SessionLayer};
use http_body::Limited;
use tower_http::{
    limit::{RequestBodyLimit, RequestBodyLimitLayer},
    trace::TraceLayer,
};

// pub fn routes(app: Arc<App>) -> Router<Limited<Body>> {
pub fn routes(app: Arc<App>) -> Router {
    let session_layer = SessionLayer::new(
        MemoryStore::new(),
        &(0..64).map(|_| rand::random::<u8>()).collect::<Vec<_>>(),
    );

    let create_post = handlers::create_post;

    let hidden = Router::new()
        .route("/login", get(handlers::get_login).post(handlers::login))
        .route("/signup", get(handlers::get_signup).post(handlers::signup))
        .layer(session_layer)
        .route_layer(middleware::from_fn(authenticate))
        .route("/recent", get(handlers::recent))
        .route("/captcha", get(handlers::captcha))
        .route("/mod", get(handlers::get_mod));

    Router::new()
        .merge(SpaRouter::new("/static", "ui/static"))
        .route("/", get(handlers::get_root))
        .route("/:board", get(handlers::get_board))
        .route("/:board", post(create_post))
        .route("/:board/:id", get(handlers::get_post))
        .nest("/.toki", hidden)
        .layer(TraceLayer::new_for_http())
        // .layer(DefaultBodyLimit::disable())
        // .layer(RequestBodyLimitLayer::new(
        //     (app.config.security.validate_upload_limit()).unwrap(),
        // ))
        .layer(Extension(app))
        .fallback(handlers::not_found.into_service())

    // .route("/.toki/captcha", get(handlers::get_post))
}
