use axum::{
    Json,
    extract::{Query, State},
    response::Response,
    routing::post,
};
use cs25_303_core::{
    database::{
        CSPageParams, PaginatedResponse,
        queries::{ItemOrArray, NumberQuery},
        red_cap::participants::{
            ResearcherQuery, ResearcherQueryBloodPressure, ResearcherQueryResult,
        },
    },
    red_cap::{EducationLevel, PreferredLanguage, Programs},
};
use tracing::instrument;
use utoipa::OpenApi;

use crate::{
    app::{SiteState, authentication::Authentication, error::InternalError},
    utils::{builder::ResponseBuilder, json::JsonBody},
};

#[derive(OpenApi)]
#[openapi(
    paths(query),
    components(
        schemas(CSPageParams,
         ResearcherQuery,
         ResearcherQueryResult,
         PaginatedResponse<ResearcherQueryResult>,
         NumberQuery<i16>,
         PreferredLanguage,
         EducationLevel,
         Programs,
         ItemOrArray<i32>,
         ResearcherQueryBloodPressure
        )
    ),
)]
pub struct ResearcherAPI;

pub fn researcher_routes() -> axum::Router<SiteState> {
    axum::Router::new().route("/query", post(query))
}
/// Query for participants that match the given query
#[utoipa::path(
    post,
    path = "/query",
    params(
        CSPageParams,
    ),
    request_body(content = ResearcherQuery, content_type = "application/json"),
    responses(
        (status = 200, description = "Participants Found", body = PaginatedResponse<ResearcherQueryResult>)
    ),
    security(
        ("session" = []),
    )
)]
#[instrument]
pub async fn query(
    State(site): State<SiteState>,
    Query(page): Query<CSPageParams>,
    auth: Authentication,
    JsonBody(participant): JsonBody<ResearcherQuery>,
) -> Result<Response, InternalError> {
    let participants = participant.query(page.into(), &site.database).await?;

    Ok(ResponseBuilder::ok().json(&participants))
}
