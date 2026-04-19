use animus_db::persona_repo::PersonaRepo;

#[derive(Clone)]
pub struct AppState {
    pub personas: PersonaRepo,
}
