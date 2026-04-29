pub mod ollama;
pub mod prompt;

pub use animus_core::persona::Persona;
pub use ollama::{OllamaClient, OllamaError};
pub use prompt::OllamaMessage;
pub use prompt::build_prompt;
pub use prompt::resolve_placeholders;
