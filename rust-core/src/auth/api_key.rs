use super::AuthProvider;

pub struct ApiKeyProvider {
    pub api_key: String,
}

impl ApiKeyProvider {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }
}

impl AuthProvider for ApiKeyProvider {
    fn authenticate(&self) -> Result<String, super::AuthError> {
        Ok(self.api_key.clone())
    }

    fn refresh(&self, _token: &str) -> Result<String, super::AuthError> {
        Ok(self.api_key.clone())
    }
}
