/// Global application settings persisted in the local database.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    /// Display name for the local user.
    pub user_name: String,
    /// Default Ollama model to use when no persona-level model is set.
    pub default_model: String,
}
