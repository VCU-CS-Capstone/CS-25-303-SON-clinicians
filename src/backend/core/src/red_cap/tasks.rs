pub mod from;
pub mod push;
pub use from::*;

use crate::database::DBError;

use super::{api::RedCapAPIError, converter::RedCapConverterError, flatten_data_to_red_cap_format};
#[derive(Debug, thiserror::Error)]
pub enum RedCapTaskError {
    #[error(transparent)]
    DatabaseError(#[from] DBError),
    #[error(transparent)]
    RedCapError(#[from] RedCapAPIError),
    #[error(transparent)]
    RedCapConversionError(#[from] RedCapConverterError),

    #[error("Participant not found")]
    ParticipantNotFound,

    #[error("Participant base information not pushed to red cap")]
    ParticipantBaseNotPushed,
    #[error("{0}")]
    Other(&'static str),
}
