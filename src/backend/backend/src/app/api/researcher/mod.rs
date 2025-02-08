use axum::{
    extract::{Query, State},
    response::Response,
    routing::post,
    Json,
};
use cs25_303_core::database::{
    red_cap::participants::{ParticipantLookup, ParticipantLookupQuery},
    tools::{PageParams, PaginatedResponse},
};
use tracing::instrument;
use utoipa::OpenApi;

use crate::app::{authentication::Authentication, error::InternalError, SiteState};

#[derive(OpenApi)]
#[openapi(
    paths(query),
    components(schemas(PageParams, ParticipantLookup, ParticipantLookupQuery, PaginatedResponse<ParticipantLookup>)),


)]
pub struct ResearcherAPI;

pub fn researcher_routes() -> axum::Router<SiteState> {
    axum::Router::new().route("/query", post(query))
}
/// Look up participants
#[utoipa::path(
    post,
    path = "/query",
    params(
        PageParams,
    ),
    request_body(content = ParticipantLookupQuery, content_type = "application/json"),
    responses(
        (status = 200, description = "Participants Found", body = PaginatedResponse<ParticipantLookup>)
    ),
    security(
        ("session" = []),
    )
)]
#[instrument]
pub async fn query(
    State(site): State<SiteState>,
    Query(page): Query<PageParams>,
    auth: Authentication,
    Json(participant): Json<ParticipantLookupQuery>,
) -> Result<Response, InternalError> {
    let participants = participant.find(page, &site.database).await?;

    todo!()
}
