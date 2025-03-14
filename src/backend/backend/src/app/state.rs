use std::{fmt::Debug, ops::Deref, sync::Arc};

use axum::extract::State;
use cs25_303_core::user::auth::AuthenticationProvidersConfig;
use http::HeaderName;
use opentelemetry::{
    global,
    metrics::{Histogram, Meter, UpDownCounter},
};
use sqlx::PgPool;
use tokio::{sync::Mutex, task::JoinHandle};
use tracing::info;

use crate::{
    config::{EnabledFeatures, robots::RobotsConfig},
    utils::ip_addr::HasForwardedHeader,
};
pub static X_FORWARDED_FOR_HEADER: HeaderName = HeaderName::from_static("x-forwarded-for");

use super::authentication::session::SessionManager;
/// The Inner State of the Website.
///
/// This part will be wrapped in an Arc to allow for sharing between different parts of the website and threads
pub struct SiteStateInner {
    pub authentication: AuthenticationProvidersConfig,
    pub session: SessionManager,
    session_cleaner: Mutex<Option<JoinHandle<()>>>,
    pub features: EnabledFeatures,
    pub metrics: AppMetrics,
    pub robots: RobotsConfig,
}
impl SiteStateInner {
    async fn set_session_cleaner(&self, handle: JoinHandle<()>) {
        let mut session_cleaner = self.session_cleaner.lock().await;
        *session_cleaner = Some(handle);
    }
    async fn take_session_cleaner(&self) -> Option<JoinHandle<()>> {
        let mut session_cleaner = self.session_cleaner.lock().await;
        session_cleaner.take()
    }
}
impl Debug for SiteStateInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SiteStateInner")
            .field("authentication", &self.authentication)
            .field("session", &self.session)
            .finish()
    }
}
impl SiteStateInner {
    pub fn new(
        authentication: AuthenticationProvidersConfig,
        session: SessionManager,
        features: EnabledFeatures,
        robots: RobotsConfig,
    ) -> Self {
        Self {
            authentication,
            session,
            features,
            session_cleaner: Mutex::new(None),
            metrics: AppMetrics::default(),
            robots,
        }
    }
}
#[derive(Debug, Clone)]
pub struct AppMetrics {
    pub meter: Meter,
    pub request_size_bytes: Histogram<u64>,
    pub response_size_bytes: Histogram<u64>,
    pub request_duration: Histogram<f64>,
    pub active_sessions: UpDownCounter<i64>,
}
impl Default for AppMetrics {
    fn default() -> Self {
        let meter = global::meter("axum-request-metrics");

        Self {
            active_sessions: meter
                .i64_up_down_counter("http.server.active_sessions")
                .with_description("The number of active sessions")
                .build(),
            request_size_bytes: meter
                .u64_histogram("http.server.request.body.size")
                .with_unit("by")
                .build(),
            response_size_bytes: meter
                .u64_histogram("http.server.response.body.size")
                .with_unit("by")
                .build(),
            request_duration: meter
                .f64_histogram("http.server.request.duration")
                .with_unit("s")
                .build(),
            meter,
        }
    }
}
/// The State of the Website.
///
/// For people who are not familiar with Rust this is a way to share data between different parts of the website.
/// Because Rust does not "support" global variables and this is the correct way to share data between different parts of the website.
#[derive(Clone, Debug)]
pub struct SiteState {
    /// The Inner State of the Website
    pub inner: Arc<SiteStateInner>,
    /// The Database Connection Pool
    pub database: PgPool,
}
impl Deref for SiteState {
    type Target = SiteStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
macro_rules! as_ref {
    (inner => {
        $(
            $name:ident => $ty:ty
        ),*
    }
    ) => {
        $(
            impl AsRef<$ty> for SiteState {
                fn as_ref(&self) -> &$ty {
                    &self.inner.$name
                }
            }
        )*
    };
    (
        $(
            $name:ident => $ty:ty
        ),*
    ) => {
        $(
            impl AsRef<$ty> for SiteState {
                fn as_ref(&self) -> &$ty {
                    &self.$name
                }
            }
        )*
    };
}
as_ref!(
    inner => {
        authentication => AuthenticationProvidersConfig,
        session => SessionManager,
        metrics => AppMetrics
    }
);
as_ref!(
    database => PgPool
);
pub type WrappedSiteState = State<SiteState>;
impl HasForwardedHeader for SiteState {
    fn forwarded_header(&self) -> Option<&HeaderName> {
        Some(&X_FORWARDED_FOR_HEADER)
    }
}
impl SiteState {
    /// Starts Internal SServices.
    ///
    /// ## Current Starts
    /// - Session Cleaner
    pub(super) async fn start(&self) {
        let inner_cloned = self.inner.clone();

        let result = SessionManager::start_cleaner(inner_cloned);
        if let Some(handle) = result {
            self.inner.set_session_cleaner(handle).await;
            info!("Session cleaner started");
        }
    }
    /// Closes the website.
    ///
    /// ## Current Closes
    /// - Database Connection
    /// - Session Cleaner Task
    pub(super) async fn close(self) {
        // Close the website
        let SiteState {
            database, inner, ..
        } = self;
        database.close().await;
        {
            inner.session.stop_cleaner();
            if let Some(handle) = inner.take_session_cleaner().await {
                handle.abort();
            }
        }
    }
    //TODO FIX ME
    pub fn is_debug(&self) -> bool {
        true
    }
}
