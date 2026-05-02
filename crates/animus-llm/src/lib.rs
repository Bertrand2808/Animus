pub mod ollama;
pub mod prompt;

pub use animus_core::persona::Persona;
pub use ollama::num_predict_for_char_limits;
pub use ollama::{OllamaClient, OllamaError, SamplingOptions};
pub use prompt::OllamaMessage;
pub use prompt::build_prompt;
pub use prompt::resolve_placeholders;
