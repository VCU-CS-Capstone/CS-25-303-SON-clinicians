use axum::{
    extract::{Query, State},
    response::Response,
    routing::post,
};
use cs25_303_core::database::{
    CSPageParams, PaginatedResponse,
    red_cap::debug_reports::{
        DebugParticipantSummary, goals::ParticipantsWithNoGoals,
        medications::ParticipantsWithNoMedications,
    },
};
use tracing::instrument;
use utoipa::OpenApi;

use crate::{
    app::{SiteState, authentication::Authentication, error::InternalError},
    utils::{ResponseBuilder, json::JsonBody},
};

#[derive(OpenApi)]
#[openapi(
    paths(no_goals,
        no_medications),
    components(
        schemas(
            CSPageParams,
            PaginatedResponse<DebugParticipantSummary>,
            ParticipantsWithNoGoals,
            ParticipantsWithNoMedications
        )
    ),
)]
pub struct DebugReportsApi;

pub fn debug_reports() -> axum::Router<SiteState> {
    axum::Router::new()
        .route("/no_goals", post(no_goals))
        .route("/no_medications", post(no_medications))
}

#[utoipa::path(
    post,
    path = "/no_goals",
    summary = "Fetches Participants with no goals",
    params(
        CSPageParams,
    ),
    request_body(content = ParticipantsWithNoGoals, content_type = "application/json"),
    responses(
        (status = 200, description = "Participants Found", body = PaginatedResponse<DebugParticipantSummary>)
    ),
    security(
        ("session" = []),
    )
)]
#[instrument]
pub async fn no_goals(
    State(site): State<SiteState>,
    Query(page): Query<CSPageParams>,
    auth: Authentication,
    JsonBody(query): JsonBody<ParticipantsWithNoGoals>,
) -> Result<Response, InternalError> {
    let participants = query.execute(page, &site.database).await?;

    Ok(ResponseBuilder::ok().json(&participants))
}
#[utoipa::path(
    post,
    path = "/no_medications",
    summary = "Fetches Participants with no medications",
    params(
        CSPageParams,
    ),
    request_body(content = ParticipantsWithNoMedications, content_type = "application/json"),
    responses(
        (status = 200, description = "Participants Found", body = PaginatedResponse<DebugParticipantSummary>)
    ),
    security(
        ("session" = []),
    )
)]
#[instrument]
pub async fn no_medications(
    State(site): State<SiteState>,
    Query(page): Query<CSPageParams>,
    auth: Authentication,
    JsonBody(query): JsonBody<ParticipantsWithNoMedications>,
) -> Result<Response, InternalError> {
    let participants = query.execute(page, &site.database).await?;

    Ok(ResponseBuilder::ok().json(&participants))
}
