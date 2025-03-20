use axum::{
    extract::{Path, Query, State},
    response::Response,
    routing::get,
};
use cs25_303_core::database::{
    CSPageParams, PaginatedResponse,
    red_cap::participants::{ParticipantMedications, Participants},
};
use serde::Deserialize;
use tracing::instrument;
use utoipa::{IntoParams, OpenApi};

use crate::utils::response::ResponseBuilder;
use crate::{
    app::{SiteState, authentication::Authentication, error::InternalError},
    utils::ErrorReason,
};

#[derive(OpenApi)]
#[openapi(
    paths(search_medications),
    components(schemas(ParticipantMedications,PaginatedResponse<ParticipantMedications>))
)]
pub struct ParticipantMedicationsAPI;

pub fn participant_medications() -> axum::Router<SiteState> {
    axum::Router::new().route("/{participant_id}/search", get(search_medications))
}
#[derive(Debug, Clone, Default, Deserialize, IntoParams)]
#[serde(default)]
#[into_params(parameter_in = Query)]
pub struct MedicationSearch {
    /// Medication name to optionally search for
    ///
    /// If not provided, all medications will be returned
    #[serde(with = "crate::utils::serde_sanitize_string")]
    #[param(nullable)]
    pub name: Option<String>,
}
#[utoipa::path(
    get,
    path = "/{participant_id}/search",
    summary = "Search for medications for a participant",
    params(
        ("participant_id" = i32, Path,  description = "Participant ID"),
        CSPageParams,
        MedicationSearch
    ),
    responses(
        (status = 200, description = "Medications for participant", body = PaginatedResponse<ParticipantMedications>, content_type = "application/json"),
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
    Query(params): Query<CSPageParams>,
    Query(MedicationSearch { name }): Query<MedicationSearch>,
) -> Result<Response, InternalError> {
    let medications = ParticipantMedications::search_medications(
        participant_id,
        &site.database,
        name,
        params.into(),
    )
    .await?;
    // If the response is empty and the participant does not exist return a 404
    if medications.is_empty()
        && !Participants::does_participant_id_exist(participant_id, &site.database).await?
    {
        return Ok(ResponseBuilder::not_found()
            .extension(ErrorReason::from("Participant Not Found"))
            .empty());
    }
    Ok(ResponseBuilder::ok().json(&medications))
}
