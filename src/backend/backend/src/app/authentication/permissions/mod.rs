use cs25_303_core::{
    database::user::{User, UserType},
    user::Permissions,
};
use response::MissingPermission;
use sqlx::PgPool;
use tracing::{debug, instrument};
pub mod response;

use super::AuthenticationError;

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
impl PermissionCheck for () {
    fn permissions_required() -> &'static [Permissions] {
        &[]
    }
    async fn check_permissions(_: &User, _: &PgPool) -> Result<(), AuthenticationError> {
        Ok(())
    }
}
#[allow(unused_macros)]
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
