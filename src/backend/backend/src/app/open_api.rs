use crate::app::api::{auth::AuthApi, researcher::ResearcherAPI};

use super::api::{self, admin::AdminAPI, location::LocationsAPI, participant::ParticipantAPI};
use axum::{
    Json, Router,
    response::{IntoResponse, Response},
};

use cs25_303_core::{
    database::red_cap::{
        Locations,
        case_notes::{BloodPressureType, CaseNote, queries::CaseNoteIDAndDate},
        participants::{ParticipantDemograhics, Participants, health_overview::HealthOverview},
        questions::{
            AdditionalOptionSettings, AdditionalQuestionSettings, BooleanQuestionSettings,
            FloatSettings, NumberSettings, QuestionOptions, QuestionType, TextBoxSize,
            TextQuestionSettings, queries::QuestionOverview,
        },
    },
    red_cap::{
        EducationLevel, Ethnicity, Gender, HealthInsurance, MedicationFrequency, MobilityDevice,
        PreferredLanguage, Programs, Race, SeenAtVCUHS, VisitType,
    },
};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, Http, HttpAuthScheme, SecurityScheme},
};
#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    nest(
        (path = "/api/auth", api = AuthApi, tags=["Authentication", "user"]),
        (path = "/api/participant", api = ParticipantAPI, tags=["participant"]),
        (path = "/api/location", api = LocationsAPI, tags=["location"]),
        (path = "/api/admin", api = AdminAPI, tags=["admin"]),
        (path = "/api/researcher", api = ResearcherAPI, tags=["Researcher"])
    ),
    paths(api::info),
    components(schemas(
        api::Instance,
        BloodPressureType, Locations, Gender, Race, SeenAtVCUHS, Programs, Ethnicity,HealthInsurance,EducationLevel,MobilityDevice,
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
    tags(
        (name = "Authentication", description = "Authentication API. Used for logging in and out"),
        (name = "participant", description = "Participant Information"),
        (name = "location", description = "Location Information"),
        (name = "admin", description = "Admin Information"),
        (name = "Researcher", description = "Researcher Advanced Queries")
    )
)]
pub struct ApiDoc;
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            let mut session_value = ApiKeyValue::new("session");
            session_value.description = Some(r#"A cookie with the session_id.
                However, you are also able pass it in as a header using Header Name of Authorization then putting `Session` as the schema and the session_id as the next parameter
                Authorization: Session {session_id}"#.to_string());
            components.add_security_scheme(
                "session",
                SecurityScheme::ApiKey(ApiKey::Cookie(session_value)),
            );
            components.add_security_scheme(
                "token",
                SecurityScheme::Http(
                    Http::builder()
                        .scheme(HttpAuthScheme::Bearer)
                        .description(Some("Bearer Token Or session_key"))
                        .build(),
                ),
            );
        }
    }
}
#[cfg(feature = "utoipa-scalar")]
pub fn open_api_router<S>(open_api: bool, scalar: bool) -> axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    use axum::routing::get;
    use tracing::error;
    use utoipa_scalar::{Scalar, Servable};

    let mut router = Router::new();
    if open_api {
        router = router.route("/open-api-doc-raw", get(api_docs));
        if scalar {
            router = router.merge(Scalar::with_url("/scalar", ApiDoc::openapi()));
        }
    } else if scalar {
        error!("Scalar is enabled but OpenAPI is not. Scalar will not be available.");
    }

    router
}

#[cfg(not(feature = "utoipa-scalar"))]
pub fn open_api_router<S>(open_api: bool, scalar: bool) -> axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    use axum::routing::get;
    use tracing::error;

    let mut router = Router::new();
    if open_api {
        router = router.route("/open-api-doc-raw", get(api_docs));
    }
    if scalar {
        error!("Scalar feature is not built in. Scalar will not be available.");
    }

    router
}

async fn api_docs() -> Response {
    Json(ApiDoc::openapi()).into_response()
}
