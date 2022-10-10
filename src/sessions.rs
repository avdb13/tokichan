// It is better to manually check whether a login expired on authentication rather than in a
// separate loop assuming the amount of logins is low enough.

use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use axum_sessions::extractors::WritableSession;
use chrono::Utc;

pub struct AuthState {
    username: String,
    session: WritableSession,
}

pub async fn authenticate<B>(
    req: Request<B>,
    next: Next<B>,
    mut state: AuthState,
) -> Result<Response, StatusCode> {
    match state.session.get::<chrono::DateTime<Utc>>(&state.username) {
        None => Ok(Redirect::to("/").into_response()),
        Some(x) => match x > chrono::Utc::now() {
            false => Ok(next.run(req).await),
            true => {
                state.session.remove(&state.username);
                Ok(Redirect::to("/").into_response())
            }
        },
    }
}
