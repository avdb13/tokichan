use axum::body::Body;
use http_body::Limited;
use std::hash::Hasher;
use std::sync::Arc;

use axum::extract::State;
use axum::http::header::SET_COOKIE;
use axum::http::HeaderValue;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{http::Request, middleware::Next, response::Response};
use axum::{Extension, RequestExt};
use axum_extra::extract::cookie::Cookie;
use axum_sessions::extractors::WritableSession;
use axum_sessions::{SameSite, SessionHandle};
use hmac::Mac;
use hyper::header::{CONTENT_TYPE, COOKIE};

use ripemd::{Digest, Ripemd160};

use tokio::sync::RwLock;
use tracing::info;

use crate::utils::error::RequestError;
use crate::utils::helpers::hash;
use crate::App;

use super::captcha::CaptchaService;
use super::error::AppError;
use super::templates::Input;

pub async fn flash<B>(mut req: Request<B>, next: Next<B>) -> Result<Response, AppError>
where
    B: Send + 'static,
{
    let Extension(session_handle): Extension<SessionHandle> = req.extract_parts().await?;
    let session = session_handle.read().await;
    let payload = session.get::<bool>("flash").unwrap_or(false);
    drop(session);

    req.extensions_mut().insert(payload);
    Ok(next.run(req).await)
}

pub async fn signed_in<B>(mut req: Request<B>, next: Next<B>) -> Result<Response, AppError>
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

pub async fn captcha_cookie<B>(
    Extension(cs): Extension<Arc<RwLock<CaptchaService>>>,
    mut request: Request<B>,
    next: Next<B>,
) -> Response {
    let cs = cs.read().await;
    let captcha = cs.recv().await;

    let mut hasher = Ripemd160::new();
    hasher.update(captcha.0);
    let hash = hasher.finalize();

    let result = base64::encode(hash);
    info!("generated captcha hash: {}", result);

    let cookie = Cookie::build("captcha", result)
        .path("/")
        .same_site(SameSite::Strict)
        .secure(true)
        .http_only(true)
        .finish();

    request.extensions_mut().insert(captcha.1);

    let mut response = next.run(request).await;

    response.headers_mut().append(
        SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).unwrap(),
    );

    response
}

pub async fn parse_fields(
    State(app): State<Arc<App>>,
    _session: WritableSession,
    request: Request<Body>,
    next: Next<Body>,
) -> Result<impl IntoResponse, Response> {
    let (mut parts, body) = request.into_parts();

    let bytes = hyper::body::to_bytes(body)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;

    let header = parts.headers.get(CONTENT_TYPE).unwrap();

    let boundary = multer::parse_boundary(header.to_str().unwrap()).unwrap();
    let multipart = multer::Multipart::with_reader(&*bytes, boundary);

    let input = app.models.parse_fields(multipart).await.unwrap();

    let captcha = parts
        .headers
        .get_all(COOKIE)
        .iter()
        .filter_map(|header| header.to_str().ok())
        .flat_map(|header| header.split(';'))
        .filter_map(|header| Cookie::parse_encoded(header.trim()).ok())
        .find(|cookie| cookie.name() == "captcha");

    let expected_hash = hash(input.captcha.as_bytes()).await;
    info!(
        "received captcha: {}, captcha hash: {}",
        &input.captcha,
        base64::encode(expected_hash.clone())
    );

    let request = match captcha {
        Some(captcha) if captcha.value() == base64::encode(expected_hash) => {
            parts.extensions.insert(Ok::<Input, RequestError>(input));
            Request::from_parts(parts, hyper::Body::from(bytes.clone()))
        }
        _ => {
            parts
                .extensions
                .insert(Err::<Input, RequestError>(RequestError::IncorrectCaptcha));
            Request::from_parts(parts, hyper::Body::from(bytes.clone()))
        }
    };

    Ok(next.run(request).await)
}
