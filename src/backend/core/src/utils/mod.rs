use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
#[error("Invalid variant: {0}")]
pub struct InvalidVariant(pub String);
