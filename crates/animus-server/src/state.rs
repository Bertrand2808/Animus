use animus_db::{
    persona_repo::PersonaRepo, settings_repo::SettingsRepo, summary_repo::SummaryRepo,
    ConversationRepo, MessageRepo,
};
use animus_llm::ollama::OllamaClient;

#[derive(Clone)]
pub struct AppState {
    pub personas: PersonaRepo,
    pub conversations: ConversationRepo,
    pub messages: MessageRepo,
    pub summaries: SummaryRepo,
    pub settings: SettingsRepo,
    pub ollama: OllamaClient,
    pub model_name: String,
    pub ollama_url: String,
    pub assets_dir: String,
    pub backups_dir: String,
}
