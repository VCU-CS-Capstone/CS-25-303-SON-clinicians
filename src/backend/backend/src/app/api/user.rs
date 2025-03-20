use axum::{response::Response, routing::get};
use cs25_303_core::database::user::User;
use utoipa::OpenApi;

use crate::{
    app::{
        SiteState,
        authentication::{Authentication, MeWithSession},
        error::InternalError,
    },
    utils::ResponseBuilder,
};

#[derive(OpenApi)]
#[openapi(paths(me, session), components(schemas(MeWithSession)))]
pub struct UserApi;
pub fn user_api() -> axum::Router<SiteState> {
    axum::Router::new()
        .route("/me", get(me))
        .route("/session", get(session))
}

#[utoipa::path(
    get,
    path = "/me",
    responses(
        (status = 200, description = "Current User", body = User, content_type = "application/json"),
        (status = 401, description = "Unauthorized"),
    ),
    summary = "Gets the current user",
    security(
        (),
    )
)]
async fn me(auth: Authentication) -> Result<Response, InternalError> {
    let Authentication::UserViaSession { user, .. } = auth else {
        unimplemented!("This should be a user");
    };
    Ok(ResponseBuilder::ok().json(&user))
}
#[utoipa::path(
    get,
    path = "/session",
    responses(
        (status = 200, description = "Returns the users current session", body = MeWithSession, content_type = "application/json"),
        (status = 401, description = "Unauthorized"),
    ),
    summary = "Gets the current user session",
    security(
        (),
    )
)]
async fn session(auth: Authentication) -> Result<Response, InternalError> {
    let Authentication::UserViaSession { user, session } = auth else {
        return Ok(ResponseBuilder::bad_request().empty());
    };
    Ok(ResponseBuilder::ok().json(&MeWithSession {
        user: user,
        session: session,
    }))
}
