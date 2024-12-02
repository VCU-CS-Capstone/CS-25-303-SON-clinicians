use super::api::{
    self, admin::AdminAPI, location::LocationsAPI, participant::ParticipantAPI, user::UserAPI,
};
use axum::{
    response::{IntoResponse, Response},
    Json, Router,
};

use cs25_303_core::{
    database::red_cap::{
        case_notes::{queries::CaseNoteIDAndDate, BloodPressureType, CaseNote},
        participants::{health_overview::HealthOverview, ParticipantDemograhics, Participants},
        questions::{
            queries::QuestionOverview, AdditionalOptionSettings, AdditionalQuestionSettings,
            BooleanQuestionSettings, FloatSettings, NumberSettings, QuestionOptions, QuestionType,
            TextBoxSize, TextQuestionSettings,
        },
        Locations,
    },
    red_cap::{
        DegreeLevel, Ethnicity, Gender, HealthInsurance, MedicationFrequency, MobilityDevice,
        PreferredLanguage, Programs, Race, SeenAtVCUHS, VisitType,
    },
};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, Http, HttpAuthScheme, SecurityScheme},
    Modify, OpenApi,
};
#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    nest(
        (path = "/api/user", api = UserAPI, tags=["user"]),
        (path = "/api/participant", api = ParticipantAPI, tags=["participant"]),
        (path = "/api/location", api = LocationsAPI, tags=["location"]),
        (path = "/api/admin", api = AdminAPI, tags=["admin"])
    ),
    paths(api::info),
    components(schemas(
        api::Instance,
        BloodPressureType, Locations, Gender, Race, SeenAtVCUHS, Programs, Ethnicity,HealthInsurance,DegreeLevel,MobilityDevice,
        MedicationFrequency,PreferredLanguage, CaseNote, CaseNoteIDAndDate, ParticipantDemograhics,HealthOverview,
        Participants,VisitType,
        QuestionOverview,
        QuestionType,
        QuestionOptions,
        AdditionalQuestionSettings,
        AdditionalOptionSettings,
        TextBoxSize,
        TextQuestionSettings,
        BooleanQuestionSettings,
        NumberSettings,
        FloatSettings
    )),
    tags()
)]
pub struct ApiDoc;
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            let mut session_value = ApiKeyValue::new("session");
            session_value.description = Some("Session Cookie".to_string());
            components.add_security_scheme(
                "session",
                SecurityScheme::ApiKey(ApiKey::Cookie(session_value)),
            );

            components.add_security_scheme(
                "api_token",
                SecurityScheme::Http(
                    Http::builder()
                        .scheme(HttpAuthScheme::Bearer)
                        .description(Some("API Token"))
                        .build(),
                ),
            );
        }
    }
}
#[cfg(feature = "utoipa-scalar")]
pub fn build_router<S>() -> axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    use axum::routing::get;
    use utoipa_scalar::{Scalar, Servable};

    Router::new()
        .route("/open-api-doc-raw", get(api_docs))
        .merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
}
#[cfg(not(feature = "utoipa-scalar"))]
pub fn build_router<S>() -> axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    use axum::routing::get;

    Router::new().route("/open-api-doc-raw", get(api_docs))
}
async fn api_docs() -> Response {
    Json(ApiDoc::openapi()).into_response()
}
