use std::sync::Arc;
pub mod error;
use anyhow::Context;
use authentication::session::SessionManager;
use axum::{extract::Request, routing::Router};
pub mod request_logging;
mod state;
pub mod utils;
use http::HeaderName;
use sqlx::postgres::PgConnectOptions;
pub use state::*;
pub mod authentication;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::info;
mod api;
mod open_api;
mod web;
use crate::config::FullConfig;
const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

pub(super) async fn start_web_server(config: FullConfig) -> anyhow::Result<()> {
    let FullConfig {
        web_server,
        database,
        tls,
        mode,
        log,
        auth,
    } = config;
    // Start the logger
    crate::logging::init(log, mode)?;
    info!("Starting web server");

    // Connect to database
    let pg_options: PgConnectOptions = database.try_into()?;
    let database = cs25_303_core::database::connect(pg_options, true).await?;
    info!("Connected to database");
    let session = SessionManager::new(None, mode)?;
    // Create the website state
    let inner = SiteStateInner::new(auth, session);
    let website = SiteState {
        inner: Arc::new(inner),
        database,
    };
    website.start().await;
    info!("Website Configured");
    let mut router = Router::new()
        .nest("/api", api::api_routes())
        .with_state(website.clone());
    if web_server.open_api_routes {
        router = router.merge(open_api::build_router())
    }
    router = router
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(request_logging::make_span)
                .on_request(|request: &Request<_>, span: &tracing::Span| {
                    request_logging::on_request(request, span);
                })
                .on_failure(request_logging::on_failure)
                .on_response(request_logging::on_response),
        )
        .layer(PropagateRequestIdLayer::new(REQUEST_ID_HEADER))
        .layer(SetRequestIdLayer::new(REQUEST_ID_HEADER, MakeRequestUuid))
        .layer(authentication::api_middleware::AuthenticationLayer(
            website.clone(),
        ));
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
