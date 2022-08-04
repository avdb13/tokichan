use axum_extra::routing::SpaRouter;
use axum::{
    handler::Handler,
    routing::get,
    Router,
};
use crate::handlers;

pub fn init_app() -> Router {
    Router::new()
        .merge(SpaRouter::new("/static", "ui/static"))

        .route("/", get(handlers::get_root))
        .route("/.toki/recent", get(handlers::recent))
        .route("/.toki/captcha", get(handlers::captcha))

        .route("/:board", get(handlers::get_board).post(handlers::create_post))
        .route("/:board/:id", get(handlers::get_post))

        .fallback(handlers::fallback.into_service())

        // .route("/.toki/captcha", get(handlers::get_post))
}
