use crate::database::prelude::*;
use crate::user::Permissions;

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct RolePermissions {
    pub id: i32,
    pub role_id: i32,
    pub scope: Permissions,
    pub created_at: DateTime<FixedOffset>,
}
#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Roles {
    pub id: i32,
    pub role_name: String,
    pub description: Option<String>,
    pub created_at: DateTime<FixedOffset>,
}
#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct UserRoles {
    pub id: i32,
    pub user_id: i32,
    pub role_id: i32,
    pub created_on: DateTime<FixedOffset>,
}
