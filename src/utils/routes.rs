use std::{sync::Arc, time::Duration};

use super::{handlers, middleware::flash};
use crate::App;

use axum::{
    error_handling::HandleErrorLayer,
    middleware,
    routing::{get, post},
    Router,
};
use axum_extra::routing::SpaRouter;
use axum_sessions::{async_session::MemoryStore, SessionLayer};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

// pub fn routes(app: Arc<App>) -> Router<Limited<Body>> {
pub fn routes(app: Arc<App>) -> Router {
    // not working (?)
    // let timeout_layer = ServiceBuilder::new()
    //     .layer(HandleErrorLayer::new(handlers::timeout))
    //     .timeout(Duration::from_secs(1));

    let store = MemoryStore::new();
    let secret = &(0..64).map(|_| rand::random::<u8>()).collect::<Vec<_>>();
    let session_layer = SessionLayer::new(store, secret).with_cookie_name("tokichan");
    let flash = middleware::from_fn(flash);

    let hidden = Router::new()
        .route("/login", get(handlers::get_login).post(handlers::login))
        .route("/signup", get(handlers::get_signup).post(handlers::signup))
        .route("/logout", get(handlers::logout))
        .route("/mod", get(handlers::get_mod))
        .route("/recent", get(handlers::get_recent))
        .route("/captcha", get(handlers::captcha));

    Router::new()
        .merge(SpaRouter::new("/tmp", ".tmp"))
        .route("/", get(handlers::get_root))
        .route("/:board/", get(handlers::get_board))
        .route("/:board/", post(handlers::create_post))
        .route("/:board/:id", get(handlers::get_post))
        .nest("/.toki", hidden)
        .route_layer(flash)
        .layer(session_layer)
        .with_state(app)
        // .layer(timeout_layer.into_inner())
        .layer(TraceLayer::new_for_http())
    // .layer(DefaultBodyLimit::disable())
    // .layer(RequestBodyLimitLayer::new(
    //     (app.config.security.validate_upload_limit()).unwrap(),
    // ))
    // .fallback(handlers::not_found.into_service())

    // .route("/.toki/captcha", get(handlers::get_post))
}
