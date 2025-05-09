use crate::utils::ErrorReason;
use crate::{app::authentication::Authentication, utils::json::JsonBody};
pub mod case_note;
pub mod goals;
pub mod medications;
pub mod stats;
use crate::utils::response::ResponseBuilder;
use axum::{
    extract::{Path, Query, State},
    response::Response,
    routing::{get, post},
};
use cs25_303_core::database::red_cap::participants::ParticipantDemograhicsResponse;
use cs25_303_core::database::red_cap::participants::health_overview::HealthOverviewResult;
use cs25_303_core::database::{
    CSPageParams, PaginatedResponse,
    red_cap::participants::{
        ParticipantLookup, ParticipantLookupQuery, ParticipantType, Participants,
    },
};

use serde::Serialize;
use tracing::instrument;
use utoipa::{OpenApi, ToSchema};

use crate::app::{SiteState, error::InternalError};

#[derive(OpenApi)]
#[openapi(
    paths(look_up_participant, get_participants,get_health_overview, get_demographics),
    components(schemas(CSPageParams, ParticipantLookup, ParticipantLookupQuery, PaginatedResponse<ParticipantLookup>, Participants, HealthOverviewResult, ParticipantDemograhicsResponse, ParticipantPartNotFound)),
    nest(
        (path = "/case_notes", api = case_note::CaseNoteAPI, tags=["Participant Case Notes"]),
        (path = "/stats", api = stats::ParticipantStatAPI, tags=["Participant Statistics"]),
        (path = "/goals", api = goals::ParticipantGoalsAPI, tags=[ "goals"]),
        (path = "/medications", api = medications::ParticipantMedicationsAPI, tags=["medications"])
    ),
    tags(
        (name = "medications", description = "Medications API"),
        (name = "goals", description = "Goals API"),
        (name = "Participant Statistics", description = "Statisitcal Information on Participants and their health"),
        (name = "Participant Case Notes", description = "Case Notes for Participants"),
    )
)]
pub struct ParticipantAPI;

pub fn participant_routes() -> axum::Router<SiteState> {
    axum::Router::new()
        .route("/lookup", post(look_up_participant))
        .route("/get/{id}", get(get_participants))
        .route("/get/{id}/health_overview", get(get_health_overview))
        .route("/get/{id}/demographics", get(get_demographics))
        .nest("/case_notes", case_note::case_note_routes())
        .nest("/stats", stats::stat_routes())
        .nest("/goals", goals::participant_goals())
        .nest("/medications", medications::participant_medications())
}
/// Look up participants
#[utoipa::path(
    post,
    path = "/lookup",
    params(
        CSPageParams,
    ),
    request_body(content = ParticipantLookupQuery, content_type = "application/json"),
    responses(
        (status = 200, description = "Participants Found", body = PaginatedResponse<ParticipantLookup>, content_type = "application/json")
    ),
    security(
        ("session" = []),
    )
)]
#[instrument]
pub async fn look_up_participant(
    State(site): State<SiteState>,
    Query(page): Query<CSPageParams>,
    auth: Authentication,
    JsonBody(participant): JsonBody<ParticipantLookupQuery>,
) -> Result<Response, InternalError> {
    let participants = participant.find(page, &site.database).await?;
    Ok(ResponseBuilder::ok().json(&participants))
}
/// Gets a participant by ID
#[utoipa::path(
    get,
    path = "/get/{id}",
    params(
        ("id", Path,  description = "Participant ID"),
    ),
    responses(
        (status = 200, description = "Participants Found", body = Participants, content_type = "application/json"),
        (status = 404, description = "Participant Not Found")
    ),
    security(
        ("session" = []),

    )
)]
#[instrument]
pub async fn get_participants(
    State(site): State<SiteState>,
    Path(id): Path<i32>,
    auth: Authentication,
) -> Result<Response, InternalError> {
    let participant = Participants::find_by_id(id, &site.database).await?;

    match participant {
        Some(participant) => Ok(ResponseBuilder::ok().json(&participant)),
        None => Ok(ResponseBuilder::not_found()
            .extension(ErrorReason::from("Participant Not Found"))
            .empty()),
    }
}
/// Used in querying other parts of a participants information
#[derive(Debug, Serialize, ToSchema)]
pub struct ParticipantPartNotFound {
    /// Rather or not the participant exists
    pub participant_exists: bool,
}
/// Gets a participant's health overview
#[utoipa::path(
    get,
    path = "/get/{id}/health_overview",
    params(
        ("id", Path,  description = "Participant ID"),
    ),
    responses(
        (status = 200, description = "Participants Found", body = HealthOverviewResult, content_type = "application/json"),
        (status = 404, description = "Participant  Health Overview Not Found", body = ParticipantPartNotFound, content_type = "application/json")
    ),
    security(
        ("session" = []),

    )
)]
#[instrument]
pub async fn get_health_overview(
    State(site): State<SiteState>,
    Path(id): Path<i32>,
    auth: Authentication,
) -> Result<Response, InternalError> {
    let health_overview = HealthOverviewResult::find_by_participant_id(id, &site.database).await?;

    match health_overview {
        Some(health_overview) => Ok(ResponseBuilder::ok().json(&health_overview)),
        None => {
            let participant_exists =
                Participants::does_participant_id_exist(id, &site.database).await?;
            let reason = if participant_exists {
                "Participant Health Overview Not Found"
            } else {
                "Participant Not Found"
            };
            Ok(ResponseBuilder::not_found()
                .extension(ErrorReason::from(reason))
                .json(&ParticipantPartNotFound { participant_exists }))
        }
    }
}
/// Gets a participant's demographics
#[utoipa::path(
    get,
    path = "/get/{id}/demographics",
    params(
        ("id", Path,  description = "Participant ID"),
    ),
    responses(
        (status = 200, description = "Participants Found", body = ParticipantDemograhicsResponse, content_type = "application/json"),
        (status = 404, description = "Participant Demographics Not Found", body = ParticipantPartNotFound, content_type = "application/json")
    ),
    security(
        ("session" = []),

    )
)]
#[instrument]
pub async fn get_demographics(
    State(site): State<SiteState>,
    Path(id): Path<i32>,
    auth: Authentication,
) -> Result<Response, InternalError> {
    let demographics =
        ParticipantDemograhicsResponse::find_by_participant_id(id, &site.database).await?;

    match demographics {
        Some(health_overview) => Ok(ResponseBuilder::ok().json(&health_overview)),
        None => {
            let participant_exists =
                Participants::does_participant_id_exist(id, &site.database).await?;
            let reason: &str = if participant_exists {
                "Participant Demographics Not Found"
            } else {
                "Participant Not Found"
            };
            Ok(ResponseBuilder::not_found()
                .extension(reason)
                .json(&ParticipantPartNotFound { participant_exists }))
        }
    }
}
