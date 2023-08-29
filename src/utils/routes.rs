use std::{sync::Arc, time::Duration};

use super::{
    captcha::CaptchaService,
    handlers,
    middleware::{captcha_cookie, parse_fields, signed_in},
};
use crate::App;

use axum::body::Body;
use axum::{
    error_handling::HandleErrorLayer,
    extract::DefaultBodyLimit,
    middleware,
    routing::{get, post},
    Extension, Router,
};
use axum_extra::routing::SpaRouter;
use axum_sessions::{async_session::MemoryStore, SessionLayer};
use tokio::sync::RwLock;
use tower::{Layer, ServiceBuilder};
use tower_http::trace::TraceLayer;

pub fn routes(app: Arc<App>, cs: Arc<RwLock<CaptchaService>>) -> Router {
    let store = MemoryStore::new();
    let secret: Vec<u8> = (0..64).map(|_| rand::random()).collect();
    let session_layer = SessionLayer::new(store, &secret).with_cookie_name("tokichan");

    let timeout_layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handlers::timeout))
        .timeout(Duration::from_millis(5000));

    let parse_fields =
        ServiceBuilder::new().layer(middleware::from_fn_with_state(app.clone(), parse_fields));

    let hidden = Router::new()
        .route("/login", get(handlers::get_login).post(handlers::login))
        .route("/signup", get(handlers::get_signup).post(handlers::signup))
        .route("/logout", get(handlers::logout))
        .route("/mod", get(handlers::get_mod))
        .route("/recent", get(handlers::get_recent))
        .route(
            "/captcha",
            get(handlers::captcha)
                .layer(middleware::from_fn(captcha_cookie))
                .route_layer(Extension(cs)),
        );

    Router::new()
        .layer(DefaultBodyLimit::disable())
        .merge(SpaRouter::new("/tmp", ".tmp"))
        .route("/", get(handlers::get_root))
        .route("/:board/", get(handlers::get_board))
        .route("/:board/", post(handlers::create_post).layer(parse_fields))
        .route("/:board/:id", get(handlers::get_post))
        .nest("/.toki", hidden)
        .route_layer(middleware::from_fn(signed_in))
        .with_state(app)
        .layer(DefaultBodyLimit::max(1024))
        .layer(session_layer)
        .layer(timeout_layer.into_inner())
        .layer(TraceLayer::new_for_http())
}
