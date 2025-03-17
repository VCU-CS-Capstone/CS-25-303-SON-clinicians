use std::sync::Arc;
pub mod error;
use anyhow::Context;
use authentication::{api_middleware::AuthenticationLayer, session::SessionManager};
use axum::{
    extract::{Request, State},
    response::Response,
    routing::Router,
};
pub mod request_logging;
mod state;
use http::Uri;
use request_logging::AppTracingLayer;
use serde::{Serialize, ser::SerializeStruct};
use sqlx::postgres::PgConnectOptions;
pub use state::*;
pub mod authentication;

use tracing::info;
mod api;
mod open_api;
mod web;
use crate::{
    config::FullConfig,
    utils::{ErrorReason, ResponseBuilder, api_error_response::APIErrorResponse},
};

pub(super) async fn start_web_server(config: FullConfig) -> anyhow::Result<()> {
    let FullConfig {
        web_server,
        database,
        tls,
        mode,
        log,
        auth,
        enabled_features,
        robots,
    } = config;
    // Start the logger
    crate::logging::init(log)?;
    info!("Starting web server");

    // Connect to database
    let pg_options: PgConnectOptions = database.try_into()?;
    let database = cs25_303_core::database::connect(pg_options, true).await?;
    info!("Connected to database");
    let session = SessionManager::new(None, mode)?;
    // Create the website state
    let inner = SiteStateInner::new(auth, session, enabled_features.clone(), robots);
    let website = SiteState {
        inner: Arc::new(inner),
        database,
    };
    website.start().await;
    info!("Website Configured");
    let router = Router::new()
        .route("/robots.txt", axum::routing::get(robots_txt))
        .nest("/api", api::api_routes())
        .merge(open_api::open_api_router(
            enabled_features.open_api_routes,
            enabled_features.scalar,
        ))
        .fallback(route_not_found)
        .layer(AuthenticationLayer(website.clone()))
        .layer(AppTracingLayer(website.clone()))
        .with_state(website.clone());

    info!("Router Configured");
    // Start the web server
    let tls = tls
        .map(|tls| {
            web::rustls_server_config(tls.private_key, tls.certificate_chain)
                .context("Failed to create TLS configuration")
        })
        .transpose()?;
    if let Some(_tls) = tls {
        todo!("Start the web server with TLS");
    } else {
        info!("Starting web server without TLS");
        web::start(web_server.bind_address, web_server.port, router, website).await?;
    }
    Ok(())
}

async fn robots_txt(State(website): State<SiteState>) -> axum::response::Response {
    website.robots.response()
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
pub async fn route_not_found(request: Request) -> Response {
    let response: APIErrorResponse<RouteNotFound, ()> = APIErrorResponse {
        message: "Not Found".into(),
        details: Some(RouteNotFound {
            uri: request.uri().clone(),
            method: request.method().clone(),
        }),
        ..Default::default()
    };
    ResponseBuilder::not_found()
        .extension(ErrorReason::from("Route not found"))
        .json(&response)
}
