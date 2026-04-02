use crate::db::{get_config_dao, get_session_dao};
use crate::session::SessionConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub editor_mode: String,
    #[serde(default)]
    pub auto_updates: bool,
    #[serde(default)]
    pub has_completed_onboarding: bool,
    #[serde(default)]
    pub num_startups: i32,
    #[serde(default)]
    pub install_method: Option<String>,
    #[serde(default)]
    pub api_key_helper: Option<String>,
    #[serde(default)]
    pub aws_auth_refresh: Option<String>,
    #[serde(default)]
    pub aws_credential_export: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            editor_mode: "emacs".to_string(),
            auto_updates: true,
            has_completed_onboarding: false,
            num_startups: 0,
            install_method: None,
            api_key_helper: None,
            aws_auth_refresh: None,
            aws_credential_export: None,
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    #[serde(default)]
    pub mcp_context_uris: Vec<String>,
    #[serde(default)]
    pub mcp_servers: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub has_trust_dialog_accepted: bool,
    #[serde(default)]
    pub has_completed_project_onboarding: bool,
    #[serde(default)]
    pub project_onboarding_seen_count: i32,
    #[serde(default)]
    pub enabled_mcp_servers: Vec<String>,
    #[serde(default)]
    pub disabled_mcp_servers: Vec<String>,
    #[serde(default)]
    pub example_files: Vec<String>,
}

pub struct ConfigManager {
    project_dir: Option<String>,
}

impl ConfigManager {
    pub fn new(project_dir: Option<&str>) -> Self {
        Self {
            project_dir: project_dir.map(String::from),
        }
    }

    pub async fn load_global_config(&self) -> Result<GlobalConfig, ConfigError> {
        let config_dao = get_config_dao().map_err(|e| ConfigError::DatabaseError(e))?;
        
        let mut config = GlobalConfig::default();
        
        let theme = config_dao.get_global_config("theme")?;
        if let Some(val) = theme {
            config.theme = val;
        }
        
        let editor_mode = config_dao.get_global_config("editor_mode")?;
        if let Some(val) = editor_mode {
            config.editor_mode = val;
        }
        
        let auto_updates = config_dao.get_global_config("auto_updates")?;
        if let Some(val) = auto_updates {
            config.auto_updates = val.parse().unwrap_or(true);
        }
        
        let has_completed_onboarding = config_dao.get_global_config("has_completed_onboarding")?;
        if let Some(val) = has_completed_onboarding {
            config.has_completed_onboarding = val.parse().unwrap_or(false);
        }
        
        let num_startups = config_dao.get_global_config("num_startups")?;
        if let Some(val) = num_startups {
            config.num_startups = val.parse().unwrap_or(0);
        }
        
        let api_key_helper = config_dao.get_global_config("api_key_helper")?;
        if let Some(val) = api_key_helper {
            config.api_key_helper = Some(val);
        }

        Ok(config)
    }

    pub async fn save_global_config(&self, config: &GlobalConfig) -> Result<(), ConfigError> {
        let config_dao = get_config_dao().map_err(|e| ConfigError::DatabaseError(e))?;
        
        config_dao.set_global_config("theme", &config.theme)?;
        config_dao.set_global_config("editor_mode", &config.editor_mode)?;
        config_dao.set_global_config("auto_updates", &config.auto_updates.to_string())?;
        config_dao.set_global_config("has_completed_onboarding", &config.has_completed_onboarding.to_string())?;
        config_dao.set_global_config("num_startups", &config.num_startups.to_string())?;
        
        if let Some(ref helper) = config.api_key_helper {
            config_dao.set_global_config("api_key_helper", helper)?;
        }

        Ok(())
    }

    pub async fn load_project_config(&self) -> Result<ProjectConfig, ConfigError> {
        let project_dir = self.project_dir.as_ref()
            .ok_or_else(|| ConfigError::NoProjectDir)?;
        
        let config_dao = get_config_dao().map_err(|e| ConfigError::DatabaseError(e))?;
        
        let mut config = ProjectConfig::default();
        
        if let Some(val) = config_dao.get_project_config("allowed_tools", project_dir)? {
            config.allowed_tools = serde_json::from_str(&val).unwrap_or_default();
        }
        
        if let Some(val) = config_dao.get_project_config("mcp_context_uris", project_dir)? {
            config.mcp_context_uris = serde_json::from_str(&val).unwrap_or_default();
        }
        
        if let Some(val) = config_dao.get_project_config("has_trust_dialog_accepted", project_dir)? {
            config.has_trust_dialog_accepted = val.parse().unwrap_or(false);
        }

        Ok(config)
    }

    pub async fn save_project_config(&self, config: &ProjectConfig) -> Result<(), ConfigError> {
        let project_dir = self.project_dir.as_ref()
            .ok_or_else(|| ConfigError::NoProjectDir)?;
        
        let config_dao = get_config_dao().map_err(|e| ConfigError::DatabaseError(e))?;
        
        config_dao.set_project_config(
            "allowed_tools",
            &serde_json::to_string(&config.allowed_tools)?,
            project_dir,
        )?;
        
        config_dao.set_project_config(
            "mcp_context_uris",
            &serde_json::to_string(&config.mcp_context_uris)?,
            project_dir,
        )?;
        
        config_dao.set_project_config(
            "has_trust_dialog_accepted",
            &config.has_trust_dialog_accepted.to_string(),
            project_dir,
        )?;

        Ok(())
    }

    pub async fn set_api_key(&self, provider: &str, key: &str) -> Result<(), ConfigError> {
        let config_dao = get_config_dao().map_err(|e| ConfigError::DatabaseError(e))?;
        config_dao.set_api_key(provider, key)?;
        Ok(())
    }

    pub async fn get_api_key(&self, provider: &str) -> Result<Option<String>, ConfigError> {
        let config_dao = get_config_dao().map_err(|e| ConfigError::DatabaseError(e))?;
        let key = config_dao.get_api_key(provider)?;
        Ok(key)
    }

    pub async fn clear_api_key(&self, provider: &str) -> Result<(), ConfigError> {
        let config_dao = get_config_dao().map_err(|e| ConfigError::DatabaseError(e))?;
        config_dao.delete(&format!("api_key.{}", provider), "global", None)?;
        Ok(())
    }

    pub async fn get_all_api_keys(&self) -> Result<HashMap<String, String>, ConfigError> {
        let config_dao = get_config_dao().map_err(|e| ConfigError::DatabaseError(e))?;
        let configs = config_dao.get_all("global", None)?;
        
        let mut keys = HashMap::new();
        for config in configs {
            if config.key.starts_with("api_key.") {
                let provider = config.key.strip_prefix("api_key.").unwrap_or(&config.key);
                keys.insert(provider.to_string(), config.value);
            }
        }
        
        Ok(keys)
    }

    pub async fn set_session_config(&self, session_id: &str, config: &SessionConfig) -> Result<(), ConfigError> {
        let config_dao = get_config_dao().map_err(|e| ConfigError::DatabaseError(e))?;
        
        config_dao.set(
            &format!("session.{}.api_key", session_id),
            &config.api_key,
            "session",
            None,
        )?;
        
        config_dao.set(
            &format!("session.{}.model", session_id),
            &config.model,
            "session",
            None,
        )?;
        
        config_dao.set(
            &format!("session.{}.max_tokens", session_id),
            &config.max_tokens.to_string(),
            "session",
            None,
        )?;

        Ok(())
    }

    pub async fn get_session_config(&self, session_id: &str) -> Result<SessionConfig, ConfigError> {
        let config_dao = get_config_dao().map_err(|e| ConfigError::DatabaseError(e))?;
        
        let api_key = config_dao.get(&format!("session.{}.api_key", session_id), "session", None)?
            .unwrap_or_default();
        
        let model = config_dao.get(&format!("session.{}.model", session_id), "session", None)?
            .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());
        
        let max_tokens_str = config_dao.get(&format!("session.{}.max_tokens", session_id), "session", None)?
            .unwrap_or_else(|| "4096".to_string());
        
        let max_tokens = max_tokens_str.parse().unwrap_or(4096);

        Ok(SessionConfig {
            api_key,
            model,
            max_tokens,
            provider: None,
            base_url: None,
            temperature: None,
            thinking_enabled: None,
            working_directory: None,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("No project directory set")]
    NoProjectDir,
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl From<rusqlite::Error> for ConfigError {
    fn from(err: rusqlite::Error) -> Self {
        ConfigError::DatabaseError(err.to_string())
    }
}
