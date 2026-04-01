use super::ProviderManager;

pub fn create_provider_manager() -> ProviderManager {
    ProviderManager::new()
}

pub fn register_default_providers(_manager: &ProviderManager) {
    // Providers are registered by the app using the library
    // This function provides a hook for app-level initialization
}
