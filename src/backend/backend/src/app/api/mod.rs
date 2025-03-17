use axum::{Json, extract::State};
use serde::Serialize;
use tower_http::cors::CorsLayer;
use tracing::instrument;
use utoipa::ToSchema;
pub mod admin;
pub mod auth;
pub mod location;
pub mod participant;
pub mod questions;
pub mod researcher;
pub mod user;
use crate::config::EnabledFeatures;

use super::SiteState;
#[derive(Debug, Clone, Serialize, ToSchema)]
#[schema(examples(Instance::example))]
pub struct Instance {
    /// The version of the Backend
    pub version: &'static str,
    /// The commit hash of the Backend
    pub commit: Option<&'static str>,
    /// The branch of the Backend Code
    pub branch: Option<&'static str>,
    /// When this version of the backend was commited
    #[schema(format = DateTime)]
    pub commit_time: Option<&'static str>,
    /// When this version of the backend was built
    #[schema(format = DateTime)]
    pub build_time: &'static str,
    /// Enabled Features on the Backend
    pub features: EnabledFeatures,
}

impl Instance {
    fn example() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION"),
            commit: option_env!("PROJECT_COMMIT_SHORT"),
            branch: option_env!("PROJECT_BRANCH"),
            commit_time: option_env!("PROJECT_COMMIT_TIME"),
            build_time: env!("PROJECT_BUILD_TIME"),
            features: EnabledFeatures::default(),
        }
    }
    pub fn new(state: SiteState) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION"),
            commit: option_env!("PROJECT_COMMIT_SHORT"),
            branch: option_env!("PROJECT_BRANCH"),
            commit_time: option_env!("PROJECT_COMMIT_TIME"),
            build_time: env!("PROJECT_BUILD_TIME"),
            features: state.inner.features.clone(),
        }
    }
}
pub fn api_routes() -> axum::Router<SiteState> {
    axum::Router::new()
        .route("/info", axum::routing::get(info))
        .nest("/auth", auth::auth_routes())
        .nest("/participant", participant::participant_routes())
        .nest("/location", location::location_routes())
        .nest("/admin", admin::admin_routes())
        .nest("/researcher", researcher::researcher_routes())
        .layer(CorsLayer::very_permissive())
}
#[utoipa::path(
    get,
    path = "/api/info",
    responses(
        (status = 200, description = "information about the Site", body = Instance)
    ),
    security(
        (),
        ("session" = []),

    )
)]
#[instrument]
pub async fn info(State(site): State<SiteState>) -> Json<Instance> {
    Json(Instance::new(site))
}
