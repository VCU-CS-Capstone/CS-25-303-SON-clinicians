use axum::{
    extract::{Path, Query, State},
    response::Response,
    routing::get,
};
use cs25_303_core::database::{
    red_cap::participants::{ParticipantMedications, Participants},
    tools::{PageParams, PaginatedResponse},
};
use serde::Deserialize;
use tracing::instrument;
use utoipa::{IntoParams, OpenApi};

use crate::app::{
    authentication::Authentication, error::InternalError,
    utils::response::builder::ResponseBuilder, SiteState,
};

#[derive(OpenApi)]
#[openapi(
    paths(get_participant_medications, search_medications),
    components(schemas(ParticipantMedications,PaginatedResponse<ParticipantMedications>))
)]
pub struct ParticipantMedicationsAPI;

pub fn participant_medications() -> axum::Router<SiteState> {
    axum::Router::new()
        .route("/{participant_id}/all", get(get_participant_medications))
        .route("/{participant_id}/search", get(search_medications))
}
#[utoipa::path(
    get,
    path = "/{participant_id}/all",
    summary = "Get all medications for a participant",
    description = "Returns all medications for a participant. Please use the search endpoint to get a paginated list of medications",
    params(
        ("participant_id", Path,  description = "Participant ID"),
    ),
    responses(
        (status = 200, description = "medications for participant", body = Vec<ParticipantMedications>),
        (status = 404, description = "Participant Not Found")
    ),
    security(
        ("session" = []),
    )
)]
#[instrument]
pub async fn get_participant_medications(
    State(site): State<SiteState>,
    Path(participant_id): Path<i32>,
    auth: Authentication,
) -> Result<Response, InternalError> {
    let medications =
        ParticipantMedications::get_all_participant_medications(participant_id, &site.database)
            .await?;

    if medications.is_empty()
        && !Participants::does_participant_id_exist(participant_id, &site.database).await?
    {
        return Ok(ResponseBuilder::not_found().empty());
    }

    Ok(ResponseBuilder::ok().json(&medications))
}
#[derive(Debug, Clone, Default, Deserialize, IntoParams)]
#[serde(default)]
#[into_params(parameter_in = Query)]
pub struct MedicationSearch {
    /// Medication name to optionally search for
    #[serde(with = "crate::utils::serde_sanitize_string")]
    #[param(nullable)]
    pub name: Option<String>,
}
#[utoipa::path(
    get,
    path = "/{participant_id}/search",
    summary = "Search for medications for a participant",
    params(
        ("participant_id", Path,  description = "Participant ID"),
        PageParams,
        MedicationSearch
    ),
    responses(
        (status = 200, description = "medications for participant", body = PaginatedResponse<ParticipantMedications>),
        (status = 404, description = "Participant Not Found")
    ),
    security(
        ("session" = []),
    )
)]
#[instrument]
pub async fn search_medications(
    State(site): State<SiteState>,
    Path(participant_id): Path<i32>,
    auth: Authentication,
    Query(params): Query<PageParams>,
    Query(MedicationSearch { name }): Query<MedicationSearch>,
) -> Result<Response, InternalError> {
    let medications =
        ParticipantMedications::search_medications(participant_id, &site.database, name, params)
            .await?;
    // If the response is empty and the participant does not exist return a 404
    if medications.is_empty()
        && !Participants::does_participant_id_exist(participant_id, &site.database).await?
    {
        return Ok(ResponseBuilder::not_found().empty());
    }
    Ok(ResponseBuilder::ok().json(&medications))
}
