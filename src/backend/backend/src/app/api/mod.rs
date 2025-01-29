use axum::{
    extract::{Request, State},
    response::Response,
    Json,
};
use http::Uri;
use serde::{ser::SerializeStruct, Serialize};
use tower_http::cors::CorsLayer;
use tracing::instrument;
use utoipa::ToSchema;
pub mod admin;
pub mod auth;
pub mod location;
pub mod participant;
pub mod questions;
pub mod user;
use crate::config::EnabledFeatures;

use super::{error::APIErrorResponse, utils::response::builder::ResponseBuilder, SiteState};
#[derive(Debug, Clone, Serialize, ToSchema)]
#[schema(examples(Instance::example))]
pub struct Instance {
    /// The version of the Backend
    pub version: &'static str,
    /// The commit hash of the Backend
    pub commit: &'static str,
    /// The branch of the Backend Code
    pub branch: &'static str,
    /// When this version of the backend was commited
    #[schema(format = DateTime)]
    pub commit_time: &'static str,
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
            commit: env!("PROJECT_COMMIT_SHORT"),
            branch: env!("PROJECT_BRANCH"),
            commit_time: env!("PROJECT_COMMIT_TIME"),
            build_time: env!("PROJECT_BUILD_TIME"),
            features: EnabledFeatures::default(),
        }
    }
    pub fn new(state: SiteState) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION"),
            commit: env!("PROJECT_COMMIT_SHORT"),
            branch: env!("PROJECT_BRANCH"),
            commit_time: env!("PROJECT_COMMIT_TIME"),
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
        .fallback(route_not_found)
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
        ("api_token" = []),
    )
)]
#[instrument]
pub async fn info(State(site): State<SiteState>) -> Json<Instance> {
    Json(Instance::new(site))
}
#[derive(Debug)]
pub struct RouteNotFound {
    pub uri: Uri,
    pub method: http::Method,
}
impl Serialize for RouteNotFound {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut struct_ser = serializer.serialize_struct("RouteNotFound", 2)?;
        struct_ser.serialize_field("uri", &self.uri.to_string())?;
        struct_ser.serialize_field("method", &self.method.to_string())?;
        struct_ser.end()
    }
}
async fn route_not_found(request: Request) -> Response {
    let response: APIErrorResponse<RouteNotFound, ()> = APIErrorResponse {
        message: "Not Found".into(),
        details: Some(RouteNotFound {
            uri: request.uri().clone(),
            method: request.method().clone(),
        }),
        ..Default::default()
    };
    ResponseBuilder::not_found().json(&response)
}
