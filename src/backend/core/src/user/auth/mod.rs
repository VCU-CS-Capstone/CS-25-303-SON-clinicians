use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthenticationProviders {
    /// Will use the the `user_authentication_password` table to authenticate users.
    Password,
    /// Will use the systems configured SAML provider to authenticate users.
    SAML,
}
/// What Authentications Providers are enabled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationProvidersConfig {
    /// Traditional username and password based authentication.
    pub password: Option<PasswordConfig>,
    /// SAML based authentication.
    pub saml: Option<SAMLProvider>,
}
impl Default for AuthenticationProvidersConfig {
    fn default() -> Self {
        Self {
            password: Some(Default::default()),
            saml: None,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SAMLProvider {
    // TODO: Configure SAML provider
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PasswordConfig {
    /// Should be false in production. Only here for testing.
    pub allow_basic_auth: bool,
    pub min_length: u8,
    pub max_length: u8,
    pub require_special: bool,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_number: bool,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            allow_basic_auth: false,
            min_length: 8,
            max_length: 64,
            require_special: false,
            require_uppercase: true,
            require_lowercase: true,
            require_number: true,
        }
    }
}
