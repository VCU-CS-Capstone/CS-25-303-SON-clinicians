use cs25_303_macros::Permissions;
use derive_more::derive::From;
use strum::EnumIs;
use utoipa::ToSchema;
#[derive(Debug, thiserror::Error, From)]
#[error("Invalid Scope: {0}")]
pub struct InvalidPermission(String);
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct PermissionDescription {
    pub key: Permissions,
    pub title: &'static str,
    pub description: &'static str,
    pub category: Option<&'static str>,
}
impl Default for PermissionDescription {
    fn default() -> Self {
        Self {
            key: Permissions::Admin,
            title: Default::default(),
            description: Default::default(),
            category: Default::default(),
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumIs, Permissions, ToSchema)]

pub enum Permissions {
    /// An Admin has unrestricted access to the system.
    #[permission(title = "Admin", category = "System")]
    Admin,
    /// A user who can add, update, remove users
    #[permission(key = "users:manage", title = "Manage Users", category = "System")]
    ManageUsers,
    /// Trigger A Red Cap Sync
    #[permission(key = "redcap:sync", title = "Sync Red Cap", category = "System")]
    SyncRedCap,
    /// A user who can read participants.
    ///
    /// - View Demographics
    /// - View Contact Information
    /// - View Case Notes
    #[permission(
        key = "participants:read",
        title = "View Participants",
        category = "Participants"
    )]
    ReadParticipants,
    /// A user who can update participants.
    ///
    /// - Update Demographics
    /// - Update Contact Information
    /// - Add or Update Case Notes
    /// - Add or Update Participants
    #[permission(
        key = "participants:update",
        title = "Update Participants",
        category = "Participants"
    )]
    UpdateParticipants,
    /// A user who can add, update, remove appointments
    #[permission(
        key = "schedule:manage",
        title = "Manage Schedule",
        category = "Schedule"
    )]
    ManageSchedule,
    /// A user who can view appointments
    #[permission(key = "schedule:read", title = "View Schedule", category = "Schedule")]
    ReadSchedule,
    /// Update Their Own User Information Excluding Password
    #[permission(key = "self:update", title = "Update Self", category = "Self")]
    UpdateSelf,
    /// Change, Update, ADd Their Own Password
    ///
    /// This should not be allowed in a Single Sign On System
    #[permission(key = "self:password", title = "Update Password", category = "Self")]
    UpdateSelfPassword,
}
