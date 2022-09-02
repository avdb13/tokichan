use std::sync::Arc;

use crate::{handlers, App};
use axum::{handler::Handler, routing::get, Extension, Router};
use axum_extra::routing::SpaRouter;
use tower_http::trace::TraceLayer;

pub fn routes(app: Extension<Arc<App>>) -> Router {
    Router::new()
        .merge(SpaRouter::new("/static", "ui/static"))
        .route("/", get(handlers::get_root))
        .route("/.toki/recent", get(handlers::recent))
        .route("/.toki/captcha", get(handlers::captcha))
        .route(
            "/:board",
            get(handlers::get_board).post(handlers::create_post),
        )
        .route("/:board/:id", get(handlers::get_post))
        .layer(Extension(app))
        .layer(TraceLayer::new_for_http())
        .fallback(handlers::fallback.into_service())

    // .route("/.toki/captcha", get(handlers::get_post))
}
