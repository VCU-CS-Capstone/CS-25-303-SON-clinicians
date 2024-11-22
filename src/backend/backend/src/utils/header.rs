use http::{header::ToStrError, HeaderValue};
use tracing::error;
/// Extension trait for [http::HeaderValue]

pub trait HeaderValueExt {
    /// Converts the header value to a string
    fn to_string(&self) -> Result<String, ToStrError>;
    /// Converts the header value to a string
    fn to_string_as_option(&self) -> Option<String>;
    /// Parses the header value into a type Over the [TryFrom] trait
    ///
    /// Error must be convertible from [ToStrError]
    fn parsed<T, E>(&self) -> Result<T, E>
    where
        T: TryFrom<String, Error = E>,
        E: From<ToStrError>;
}
impl HeaderValueExt for HeaderValue {
    fn to_string(&self) -> Result<String, ToStrError> {
        self.to_str().map(|x| x.to_string())
    }

    fn to_string_as_option(&self) -> Option<String> {
        self.to_str()
            .map(|x| x.to_string())
            .inspect_err(|error| {
                error!("Failed to convert header value to string: {}", error);
            })
            .ok()
    }

    fn parsed<T, E>(&self) -> Result<T, E>
    where
        T: TryFrom<String, Error = E>,
        E: From<ToStrError>,
    {
        let value = self.to_string()?;
        T::try_from(value)
    }
}
