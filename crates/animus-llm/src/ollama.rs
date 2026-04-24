use std::time::Duration;

use reqwest::Client;
use serde::Deserialize;

use crate::OllamaMessage;

/// HTTP client for the Ollama `/api/generate` endpoint.
///
/// Wraps `reqwest::Client` (internally `Arc`-backed — cheap to clone).
#[derive(Clone)]
pub struct OllamaClient {
    base_url: String,
    client: Client,
}

impl OllamaClient {
    /// Creates a new client pointing at `base_url` (e.g. `"http://localhost:11434"`).
    ///
    /// # Examples
    /// ```
    /// use animus_llm::OllamaClient;
    /// let client = OllamaClient::new("http://localhost:11434");
    /// ```
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: Client::new(),
        }
    }

    /// Sends `messages` to the given `model` and returns the assistant text.
    ///
    /// # Errors
    /// - [`OllamaError::Network`] — connection failed or timed out (30 s hard limit)
    /// - [`OllamaError::Model`]   — Ollama returned a non-2xx status
    /// - [`OllamaError::Parse`]   — response body could not be decoded
    pub async fn complete(
        &self,
        model: &str,
        messages: Vec<OllamaMessage>,
    ) -> Result<String, OllamaError> {
        let prompt = messages_to_prompt(&messages);

        let request_body = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": false,
        });

        let response = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .timeout(Duration::from_secs(30))
            .json(&request_body)
            .send()
            .await
            .map_err(OllamaError::Network)?;

        if !response.status().is_success() {
            return Err(OllamaError::Model(format!(
                "Ollama returned {}",
                response.status()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|_| OllamaError::Parse("Failed to read response body".to_string()))?;

        let parsed: OllamaResponse = serde_json::from_str(&body)
            .map_err(|_| OllamaError::Parse("Failed to parse response".to_string()))?;

        Ok(parsed.response)
    }
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
    #[allow(dead_code)]
    done: bool,
}

/// Errors produced by [`OllamaClient::complete`].
#[derive(Debug)]
pub enum OllamaError {
    /// Network failure: timeout, DNS, or connection refused.
    Network(reqwest::Error),
    /// Ollama returned a non-2xx HTTP status.
    Model(String),
    /// Response body could not be parsed.
    Parse(String),
}

impl std::fmt::Display for OllamaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OllamaError::Network(e) => write!(f, "Network error: {}", e),
            OllamaError::Model(msg) => write!(f, "Model error: {}", msg),
            OllamaError::Parse(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for OllamaError {}

fn messages_to_prompt(messages: &[OllamaMessage]) -> String {
    messages
        .iter()
        .map(|msg| format!("{}:\n{}", msg.role.to_uppercase(), msg.content))
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OllamaMessage;

    // Test 1 : création client (pas de I/O)
    #[test]
    fn new_with_base_url() {
        let _client = OllamaClient::new("http://localhost:11434");
        // juste vérifier que we can construct
        // (méthode complete() sera tokio::test donc test() suffit ici)
    }

    // Test 2 : complete() retourne non vide si Ollama répond
    // REQUIRE OLLAMA RUNNING
    #[tokio::test]
    #[ignore] // CI skip -> run en local avec : cargo test -- --ignored
    async fn complete_returns_non_empty_string() {
        let client = OllamaClient::new("http://localhost:11434");
        let messages = vec![OllamaMessage {
            role: "user".to_string(),
            content: "Hi".to_string(),
        }];

        match client.complete("mistral", messages).await {
            Ok(response) => {
                assert!(!response.is_empty());
                // Ne pas vérifier contenu exact (LLM aléatoire)
            }
            Err(OllamaError::Network(_)) => {
                // Si CI ou Ollama down → skip silencieusement
                eprintln!("Ollama not running, skipping");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    // Test 3 : erreur parsing (mock)
    // Si on peut mocker reqwest → vérifier OllamaError::Parse
    // Sinon → passer (complexe en Rust)
}
