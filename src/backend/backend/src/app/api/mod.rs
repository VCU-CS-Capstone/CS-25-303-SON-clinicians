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
use super::{error::APIErrorResponse, utils::response::builder::ResponseBuilder, SiteState};
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Instance {
    pub version: &'static str,
}
impl Instance {
    pub fn new(_state: SiteState) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION"),
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
