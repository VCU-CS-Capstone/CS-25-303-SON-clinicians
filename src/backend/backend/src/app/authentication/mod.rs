//! This module is responsible for handling the authentication of the user
//!
//! ## How this module works?
//!
//! When a request is made it will run the code within `api_middleware.rs`. Specficly [api_middleware::AuthenticationMiddleware]::call
//!
//! This will parse the request and pull specific information from the request.
//!
//! In another one of my projects it will would parse Basic Auth, API Tokens and Cookies for sessions.
//!
//! Once it is done it puts the results into [AuthenticationRaw] and says go on to the rest of the request.
//!
//! Then a request handler function requests the type [Authentication] from the request. it will call the from_request_parts function.
//! and at that point take the [AuthenticationRaw] and
//! the state of the website and return the [Authentication] type. By checking further that the authentication is valid.
//! Might make database queries or a request to another service to ensure authentication is valid.

use std::future::Future;

use super::{error::IntoErrorResponse, utils::response::MissingPermission, SiteState};
use axum::{
    body::Body,
    extract::{FromRef, FromRequestParts},
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::Cookie;
use cs25_303_core::database::{
    user::{User, UserType},
    DBError,
};
use cs25_303_core::user::Permissions;
use derive_more::derive::From;
use header::AuthorizationHeader;
use http::request::Parts;
use serde::Serialize;
use session::{Session, SessionManager};
use sqlx::PgPool;
use strum::EnumIs;
use thiserror::Error;
use tracing::{debug, error, instrument};
use utoipa::ToSchema;
pub mod api_middleware;
pub mod header;
pub mod session;
/// The user information with the session information
#[derive(Debug, Serialize, Clone, From, ToSchema)]
pub struct MeWithSession {
    /// The session information
    pub session: Session,
    /// Your user information
    pub user: User,
}

/// Possible Errors that can occur during authentication
#[derive(Error, Debug)]
pub enum AuthenticationError {
    // A generic error that can be used to return a specific error
    #[error("Error: {0}")]
    RequestTypeError(Box<dyn IntoErrorResponse>),
    /// The user is not logged in
    #[error("You are not logged in.")]
    Unauthorized,
    /// The user login is accepted but the action is forbidden with current credentials
    #[error("Forbidden")]
    Forbidden,
    #[error(transparent)]
    MissingPermission(#[from] MissingPermission),
}
impl From<DBError> for AuthenticationError {
    fn from(err: DBError) -> Self {
        AuthenticationError::RequestTypeError(Box::new(err))
    }
}
impl IntoResponse for AuthenticationError {
    fn into_response(self) -> axum::response::Response {
        error!("Authentication Error: {}", self);
        match self {
            AuthenticationError::RequestTypeError(err) => err.into_response_boxed(),
            AuthenticationError::MissingPermission(err) => err.into_response(),
            AuthenticationError::Forbidden => Response::builder()
                .status(http::StatusCode::FORBIDDEN)
                .body(Body::from(
                    "You do not have the required permissions to access this resource",
                ))
                .unwrap(),
            other => Response::builder()
                .status(http::StatusCode::UNAUTHORIZED)
                .body(Body::from(format!("Authentication Error: {}", other)))
                .unwrap(),
        }
    }
}
pub trait PermissionCheck {
    #[instrument(
        name = "PermissionCheck::check_permissions",
        skip(user, db),
        fields(project_module = "Authentication")
    )]
    fn check_permissions(
        user: &User,
        db: &PgPool,
    ) -> impl Future<Output = Result<(), AuthenticationError>> + Send {
        async move {
            let log_permission_checks = tracing::enabled!(tracing::Level::DEBUG);

            for permission in Self::permissions_required() {
                if log_permission_checks {
                    debug!("Checking permission: {:?}", permission);
                }
                if !user.has_permission(*permission, db).await? {
                    return Err(MissingPermission::from(*permission).into());
                }
            }
            if log_permission_checks {
                debug!("All permissions passed");
            }
            Ok(())
        }
    }

    fn permissions_required() -> &'static [Permissions];
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoRequiredPermissions;
impl PermissionCheck for NoRequiredPermissions {
    fn permissions_required() -> &'static [Permissions] {
        &[]
    }
    async fn check_permissions(_: &User, _: &PgPool) -> Result<(), AuthenticationError> {
        Ok(())
    }
}
macro_rules! permission_check {
    (
        $(#[$docs:meta])*
        $name:ident => $($perm:expr),+
    ) => {
        $(#[$docs])*
        pub struct $name;
        impl crate::app::authentication::auth_with_perm::PermissionCheck for $name {
            fn permissions_required() -> &'static [Permissions] {
                &[$($perm),+]
            }
        }
        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                use crate::app::authentication::auth_with_perm::PermissionCheck;
                f.debug_struct(stringify!($name))
                .field("permissions", &Self::permissions_required())
                .finish()
            }
        }
    };
}

#[derive(Clone, Debug, PartialEq, EnumIs)]
#[allow(clippy::large_enum_variant)]
pub enum Authentication<PC: PermissionCheck = NoRequiredPermissions> {
    UserViaSession { user: User, session: Session },

    Phantom(std::marker::PhantomData<PC>),
}
impl<PC: PermissionCheck> Authentication<PC> {
    /// Checks if the user has the required permission
    ///
    /// # Arguments
    /// * `state` - The state of the website used to make additional sql queries if needed
    /// * `scope` - The scope that the user needs to have
    ///
    /// ## Note
    /// This function is just a skeleton and will be replaced with a real authentication method but the structure should remain the same
    pub async fn has_permission(
        &self,
        _state: &SiteState,
        _scope: Permissions,
    ) -> Result<(), AuthenticationError> {
        Ok(())
    }
    /// Checks if the user has the required permissions
    pub async fn has_many_scopes(
        &self,
        _state: &SiteState,
        _scopes: impl Iterator<Item = Permissions>,
    ) -> Result<(), AuthenticationError> {
        Ok(())
    }
}
impl<S, PC> FromRequestParts<S> for Authentication<PC>
where
    SiteState: FromRef<S>,
    S: Send + Sync,
    PC: PermissionCheck,
{
    type Rejection = AuthenticationError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let raw_extension = parts.extensions.get::<AuthenticationRaw>().cloned();
        let state = SiteState::from_ref(state);
        match raw_extension {
            Some(AuthenticationRaw::Session(session)) => {
                let user = session.get_user(&state.database).await?;
                if let Some(user) = user {
                    PC::check_permissions(&user, &state.database).await?;

                    return Ok(Authentication::UserViaSession { user, session });
                } else {
                    error!("User not found");
                    return Err(AuthenticationError::Unauthorized);
                }
            }
            _ => {
                error!("No Authentication Data Extracted from Request");
                return Err(AuthenticationError::Unauthorized);
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, EnumIs)]
pub enum AuthenticationRaw {
    /// The user is logged in with a session
    Session(Session),
    /// No Authorization Header was passed.API Routes will most likely reject this
    NoIdentification,
}
impl AuthenticationRaw {
    fn session_cookie(session: &str, session_manager: impl AsRef<SessionManager>) -> Self {
        match session_manager.as_ref().get_session(session) {
            Ok(Some(ok)) => AuthenticationRaw::Session(ok),
            Err(err) => {
                error!("Failed to get session: {}", err);
                AuthenticationRaw::NoIdentification
            }
            Ok(None) => AuthenticationRaw::NoIdentification,
        }
    }
    pub fn new_from_cookie(cookie: &Cookie<'static>, site: &SiteState) -> Self {
        match cookie.name() {
            "session" => AuthenticationRaw::session_cookie(cookie.value(), site),
            _ => AuthenticationRaw::NoIdentification,
        }
    }
    pub fn new_from_auth_header(header: AuthorizationHeader, site: &SiteState) -> Self {
        match header {
            AuthorizationHeader::Session { session } => {
                AuthenticationRaw::session_cookie(&session, site)
            }
            _ => AuthenticationRaw::NoIdentification,
        }
    }
}

pub mod utils {
    use cs25_303_core::database::user::{
        auth::UserAndPasswordAuth,
        find_user_by_email_or_username_with_password_auth,
        login::{add_login_attempt, AdditionalFootprint},
        User,
    };
    use sqlx::{types::Uuid, PgPool};
    use tracing::{debug, instrument};

    use super::AuthenticationError;

    #[inline(always)]
    #[instrument(
        skip(username, password, database),
        fields(project_module = "Authentication")
    )]
    pub async fn verify_login(
        username: impl AsRef<str>,
        password: impl AsRef<str>,
        ip_address: String,
        additional_footprint: Option<AdditionalFootprint>,
        database: &PgPool,
    ) -> Result<(User, Uuid), AuthenticationError> {
        let user_found: Option<UserAndPasswordAuth> =
            find_user_by_email_or_username_with_password_auth(username, database)
                .await
                .map_err(|err| AuthenticationError::RequestTypeError(Box::new(err)))?;

        let Some(user_found) = user_found else {
            debug!("User not found");
            add_login_attempt(None, &ip_address, false, additional_footprint, database).await?;
            return Err(AuthenticationError::Unauthorized);
        };

        let UserAndPasswordAuth {
            user,
            password_auth,
        } = user_found;

        let Some(password_auth) = password_auth else {
            debug!(?user, "User has no password auth");
            add_login_attempt(
                Some(user.id),
                &ip_address,
                false,
                additional_footprint,
                database,
            )
            .await?;
            return Err(AuthenticationError::Unauthorized);
        };

        if let Err(err) =
            password::verify_password(password.as_ref(), password_auth.password.as_deref())
        {
            debug!("Invalid Password: {}", err);

            add_login_attempt(
                Some(user.id),
                &ip_address,
                false,
                additional_footprint,
                database,
            )
            .await?;
            return Err(err);
        }
        debug!("Login successful");
        let id = add_login_attempt(
            Some(user.id),
            &ip_address,
            true,
            additional_footprint,
            database,
        )
        .await?;
        Ok((user, id))
    }

    pub mod password {
        use argon2::{
            password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
        };
        use rand::rngs::OsRng;
        use tracing::{debug, error, instrument};

        use crate::app::authentication::AuthenticationError;
        #[instrument(skip(password), fields(project_module = "Authentication"))]
        pub fn encrypt_password(password: &str) -> Option<String> {
            let salt = SaltString::generate(&mut OsRng);

            let argon2 = Argon2::default();

            let password = argon2.hash_password(password.as_ref(), &salt);
            match password {
                Ok(ok) => Some(ok.to_string()),
                Err(err) => {
                    error!("Failed to hash password: {}", err);
                    None
                }
            }
        }
        #[instrument(skip(password, hash), fields(project_module = "Authentication"))]
        pub fn verify_password(
            password: &str,
            hash: Option<&str>,
        ) -> Result<(), AuthenticationError> {
            let argon2 = Argon2::default();
            let Some(parsed_hash) = hash.map(PasswordHash::new).transpose().map_err(|err| {
                error!("Failed to parse password hash: {}", err);
                AuthenticationError::RequestTypeError(Box::new(err))
            })?
            else {
                debug!("No password hash set");
                return Err(AuthenticationError::Unauthorized);
            };

            if argon2
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_err()
            {
                return Err(AuthenticationError::Unauthorized);
            }
            Ok(())
        }
    }
}
