use super::AuthProvider;

pub struct OAuthProvider {
    pub client_id: String,
    pub auth_url: String,
}

impl OAuthProvider {
    pub fn new(client_id: &str, auth_url: &str) -> Self {
        Self {
            client_id: client_id.to_string(),
            auth_url: auth_url.to_string(),
        }
    }
}

impl AuthProvider for OAuthProvider {
    fn authenticate(&self) -> Result<String, super::AuthError> {
        Ok(String::new())
    }

    fn refresh(&self, token: &str) -> Result<String, super::AuthError> {
        Ok(token.to_string())
    }
}
