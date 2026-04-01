pub mod oauth;
pub mod api_key;
pub mod keychain;

pub trait AuthProvider: Send + Sync {
    fn authenticate(&self) -> Result<String, AuthError>;
    fn refresh(&self, token: &str) -> Result<String, AuthError>;
}

pub struct AuthError {
    pub message: String,
}
