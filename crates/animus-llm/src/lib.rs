pub mod prompt;
pub mod ollama;

pub use animus_core::persona::Persona;
pub use prompt::OllamaMessage;
pub use prompt::build_prompt;
pub use ollama::{OllamaClient, OllamaError};
