use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    extract::cookie::{Cookie, Expiration},
    headers::UserAgent,
};
use cs25_303_core::database::user::login::AdditionalFootprint;
use http::header::SET_COOKIE;
use tracing::{debug, instrument};
use utoipa::{OpenApi, ToSchema};

use crate::{
    app::{
        SiteState,
        authentication::{Authentication, MeWithSession, utils::verify_login},
        error::InternalError,
    },
    utils::{
        ErrorReason, api_error_response::APIErrorResponse, builder::ResponseBuilder,
        ip_addr::ConnectionIpAddr, json::JsonBody, request_logging::request_id::RequestId,
    },
};

#[derive(OpenApi)]
#[openapi(
    paths(login, logout),
    components(schemas(LoginPasswordBody, MeWithSession))
)]
pub struct AuthApi;
pub fn auth_routes() -> axum::Router<SiteState> {
    axum::Router::new()
        .route("/login/password", axum::routing::post(login))
        .route("/logout", axum::routing::get(logout))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct LoginPasswordBody {
    /// The email or username of the user
    ///
    /// This field can also be called with `username` or `email`
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
    summary = "Attempt User login with a password",
    security(
        (),
    )
)]
#[instrument]
pub async fn login(
    State(site): State<SiteState>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    request_id: RequestId,
    ConnectionIpAddr(ip_addr): ConnectionIpAddr,
    JsonBody(login): JsonBody<LoginPasswordBody>,
) -> Result<Response, InternalError> {
    if site.authentication.password.is_none() {
        return Ok(ResponseBuilder::forbidden()
            .extension(ErrorReason::from("Password Authentication is not enabled"))
            .json(&APIErrorResponse::<(), ()> {
                message: "Password Authentication is not enabled".into(),
                details: None,
                error: None,
            }));
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
        ip_addr.to_string(),
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
    return Ok(ResponseBuilder::ok()
        .header(SET_COOKIE, cookie.encoded().to_string())
        .json(&user_with_session));
}
#[utoipa::path(
    get,
    summary = "Logout of the current session",
    path = "/logout",
    responses(
        (status = 201, description = "Logout successful"),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("session" = []),
    )
)]
#[instrument]
pub async fn logout(
    State(site): State<SiteState>,
    auth: Authentication,
) -> Result<Response, InternalError> {
    match auth {
        Authentication::UserViaSession { user: _, session } => {
            debug!(?session, "Logging out user");
            site.session.delete_session(&session.session_key)?;
        }
        _ => {
            return Ok(ResponseBuilder::unauthorized().empty());
        }
    }
    let cookie = Cookie::build(("session", ""))
        .secure(true)
        .path("/")
        .expires(Expiration::Session)
        .build();

    Ok(ResponseBuilder::no_content()
        .header(SET_COOKIE, cookie.encoded().to_string())
        .empty())
}
