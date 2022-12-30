use axum::{http::Request, middleware::Next, response::Response};
use axum::{Extension, RequestExt};
use axum_sessions::SessionHandle;

use super::error::AppError;

pub async fn flash<B>(mut req: Request<B>, next: Next<B>) -> Result<Response, AppError>
where
    B: Send + 'static,
{
    let Extension(session_handle): Extension<SessionHandle> = req.extract_parts().await?;
    let session = session_handle.read().await;
    let payload = session.get::<bool>("signed_in").unwrap_or(false);
    drop(session);

    req.extensions_mut().insert(payload);
    Ok(next.run(req).await)
}
