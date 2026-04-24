use animus_db::{persona_repo::PersonaRepo, ConversationRepo, MessageRepo};

#[derive(Clone)]
pub struct AppState {
    pub personas: PersonaRepo,
    pub conversations: ConversationRepo,
    pub messages: MessageRepo,
}
