use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, State},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::{
    extract::cookie::{Cookie, Expiration},
    headers::UserAgent,
    TypedHeader,
};
use cs25_303_core::database::user::login::AdditionalFootprint;
use http::{header::SET_COOKIE, StatusCode};
use tracing::instrument;
use utoipa::{OpenApi, ToSchema};

use crate::app::{
    authentication::{utils::verify_login, MeWithSession},
    error::InternalError,
    request_logging::RequestId,
    SiteState,
};

#[derive(OpenApi)]
#[openapi(paths(login), components(schemas(LoginPasswordBody, MeWithSession)))]
pub struct AuthApi;
pub fn auth_routes() -> axum::Router<SiteState> {
    axum::Router::new().route("/login/password", axum::routing::post(login))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct LoginPasswordBody {
    /// The email or username of the user
    #[serde(alias = "email", alias = "username")]
    pub email_or_username: String,
    /// The password of the user
    pub password: String,
}
/// Attempts a user login with a password.
#[utoipa::path(
    post,
    path = "/login/password",
    request_body(content = LoginPasswordBody, content_type = "application/json"),
    responses(
        (status = 200, description = "Login successful", body = MeWithSession),
        (status = 400, description = "Bad Request. Note: This request requires a User-Agent Header"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Password Authentication is not enabled"),
    ),
    security(
        (),
    )
)]
#[instrument]
pub async fn login(
    State(site): State<SiteState>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    request_id: RequestId,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(login): axum::Json<LoginPasswordBody>,
) -> Result<Response, InternalError> {
    if site.authentication.password.is_none() {
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body("Password Authentication is not enabled".into())
            .unwrap());
    }
    let LoginPasswordBody {
        email_or_username,
        password,
    } = login;
    let additional_footprint = AdditionalFootprint {
        user_agent: user_agent.to_string(),
        request_id: request_id.to_string(),
    };
    let (user, login_id) = match verify_login(
        email_or_username,
        password,
        addr.to_string(),
        Some(additional_footprint),
        &site.database,
    )
    .await
    {
        Ok(ok) => ok,
        Err(err) => {
            return Ok(err.into_response());
        }
    };

    let duration = chrono::Duration::days(1);

    let session = site.session.create_session(user.id, login_id, duration)?;
    let cookie = Cookie::build(("session", session.session_key.clone()))
        .secure(true)
        .path("/")
        .expires(Expiration::Session)
        .build();
    let user_with_session = MeWithSession::from((session.clone(), user));
    return Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .header(SET_COOKIE, cookie.encoded().to_string())
        .body(serde_json::to_string(&user_with_session)?.into())
        .unwrap());
}
