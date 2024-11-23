use crate::{
    app::{api::PaginatedResponse, authentication::Authentication},
    utils::{not_found_response, ok_json_response},
};
use axum::{
    extract::{Path, Query, State},
    response::Response,
    routing::{get, post},
    Json,
};
use cs25_303_core::database::red_cap::participants::{
    health_overview::{HealthOverview, HealthOverviewType},
    ParticipantDemograhics, ParticipantLookup, ParticipantLookupQuery, ParticipantType,
    Participants,
};
use serde::Deserialize;
use tracing::instrument;
use utoipa::{OpenApi, ToSchema};

use crate::app::{error::InternalError, SiteState};

#[derive(OpenApi)]
#[openapi(
    paths(look_up_participant, get_participants,get_health_overview, get_demographics),
    components(schemas(LookupPage, ParticipantLookup, ParticipantLookupQuery, PaginatedResponse<ParticipantLookup>, Participants, HealthOverview, ParticipantDemograhics))
)]
pub struct ParticipantAPI;

pub fn participant_routes() -> axum::Router<SiteState> {
    axum::Router::new()
        .route("/lookup", post(look_up_participant))
        .route("/get/:id", get(get_participants))
        .route("/get/:id/health_overview", get(get_health_overview))
        .route("/get/:id/demographics", get(get_demographics))
}
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(default)]
pub struct LookupPage {
    pub page_size: i32,
    pub page_number: i32,
}
impl Default for LookupPage {
    fn default() -> Self {
        Self {
            page_size: 10,
            page_number: 1,
        }
    }
}
#[utoipa::path(
    post,
    path = "/lookup",
    params(
        ("page_size" = i32, Query, description = "Number of items per page"),
        ("page_number" = i32, Query, description = "Page number"),
    ),
    responses(
        (status = 200, description = "Participants Found", body = PaginatedResponse<ParticipantLookup>)
    )
)]
#[instrument]
pub async fn look_up_participant(
    State(site): State<SiteState>,
    Query(page): Query<LookupPage>,
    Json(participant): Json<ParticipantLookupQuery>,
    // TODO auth: Authentication,
) -> Result<Response, InternalError> {
    let participants = participant.find(&site.database).await?;
    // TODO: Implement pagination

    Ok(Response::builder()
        .status(200)
        .body(serde_json::to_string(&participants)?.into())?)
}
#[utoipa::path(
    get,
    path = "/get/{id}",
    params(
        ("id", Path,  description = "Participant ID"),
    ),
    responses(
        (status = 200, description = "Participants Found", body = Participants),
        (status = 404, description = "Participant Not Found")
    )
)]
#[instrument]
pub async fn get_participants(
    State(site): State<SiteState>,
    Path(id): Path<i32>,
    // TODO auth: Authentication,
) -> Result<Response, InternalError> {
    let participant = Participants::find_by_id(id, &site.database).await?;

    match participant {
        Some(participant) => ok_json_response(participant),
        None => not_found_response(),
    }
}

#[utoipa::path(
    get,
    path = "/get/{id}/health_overview",
    params(
        ("id", Path,  description = "Participant ID"),
    ),
    responses(
        (status = 200, description = "Participants Found", body = HealthOverview),
        (status = 404, description = "Participant Not Found")
    )
)]
#[instrument]
pub async fn get_health_overview(
    State(site): State<SiteState>,
    Path(id): Path<i32>,
    // TODO auth: Authentication,
) -> Result<Response, InternalError> {
    let health_overview = HealthOverview::find_by_participant_id(id, &site.database).await?;

    match health_overview {
        Some(health_overview) => ok_json_response(health_overview),
        None => not_found_response(),
    }
}

#[utoipa::path(
    get,
    path = "/get/{id}/demographics",
    params(
        ("id", Path,  description = "Participant ID"),
    ),
    responses(
        (status = 200, description = "Participants Found", body = ParticipantDemograhics),
        (status = 404, description = "Participant Not Found")
    )
)]
#[instrument]
pub async fn get_demographics(
    State(site): State<SiteState>,
    Path(id): Path<i32>,
    // TODO auth: Authentication,
) -> Result<Response, InternalError> {
    let health_overview = HealthOverview::find_by_participant_id(id, &site.database).await?;

    match health_overview {
        Some(health_overview) => ok_json_response(health_overview),
        None => not_found_response(),
    }
}
