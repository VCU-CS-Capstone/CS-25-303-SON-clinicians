use strum::EnumIs;
use utoipa::ToSchema;

#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    Copy,
    EnumIs,
    serde::Serialize,
    serde::Deserialize,
    sqlx::Type,
    strum::Display,
    ToSchema,
)]
#[sqlx(type_name = "VARCHAR(255)")]
pub enum Scopes {
    /// An Admin has unrestricted access to the system.
    Admin,
    /// A user who can read participants.
    ///
    /// - View Demographics
    /// - View Contact Information
    /// - View Case Notes
    ReadParticipants,
    /// A user who can update participants.
    ///
    /// - Update Demographics
    /// - Update Contact Information
    /// - Add or Update Case Notes
    UpdateParticipants,
    /// A user who can add participants.
    AddParticipants,
    /// A user who can add, update, remove users
    ManageUsers,
    /// A user who can add, update, remove appointments
    ManageSchedule,
    /// A user who can view appointments
    ViewSchedule,
}
