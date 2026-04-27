use animus_db::{
    persona_repo::PersonaRepo, summary_repo::SummaryRepo, ConversationRepo, MessageRepo,
};
use animus_llm::ollama::OllamaClient;

#[derive(Clone)]
pub struct AppState {
    pub personas: PersonaRepo,
    pub conversations: ConversationRepo,
    pub messages: MessageRepo,
    pub summaries: SummaryRepo,
    pub ollama: OllamaClient,
    pub model_name: String,
}
