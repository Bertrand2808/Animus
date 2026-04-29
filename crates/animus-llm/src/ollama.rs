use crate::OllamaMessage;
use futures::Stream;
use futures::StreamExt;
use futures::TryStreamExt;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use tokio::io::AsyncBufReadExt;
use tokio_util::io::StreamReader;

/// Represents a chunk of data from the Ollama stream.
///
/// - `Token`: a regular token string from the model's output.
/// - `Done`: the stream has finished, with an optional `eval_count`.
#[derive(Debug)]
pub enum StreamChunk {
    Token(String),
    Done { eval_count: u32 },
}

/// HTTP client for Ollama API (`/api/generate` and `/api/chat` endpoints).
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

    /// Returns `true` if Ollama is reachable (GET `/api/tags` succeeds within 3 s).
    pub async fn ping(&self) -> bool {
        self.client
            .get(format!("{}/api/tags", self.base_url))
            .timeout(Duration::from_secs(3))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    pub fn stream(
        &self,
        model: &str,
        messages: Vec<OllamaMessage>,
    ) -> impl Stream<Item = Result<StreamChunk, OllamaError>> {
        let request_body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": true,
        });
        let url = format!("{}/api/chat", &self.base_url);
        let client = self.client.clone();
        async_stream::try_stream! {
            let response = client.post(&url)
                .timeout(Duration::from_secs(30))
                .json(&request_body)
                .send()
                .await
                .map_err(OllamaError::Network)?;

            if !response.status().is_success() {
                return Err(OllamaError::Model(
                    format!("Ollama returned {}", response.status())
                ))?;
            }

            let byte_stream = response.bytes_stream()
                    .map_err(std::io::Error::other);
            let reader = StreamReader::new(byte_stream);
            let mut lines_stream = tokio_stream::wrappers::LinesStream::new(reader.lines());
            while let Some(line) = lines_stream.next().await {
                let line = line.map_err(|e| OllamaError::Parse(e.to_string()))?;
                let parsed = serde_json::from_str::<OllamaStreamResponse>(&line)
                    .map_err(|e| OllamaError::Parse(e.to_string()))?;
                if parsed.done {
                    yield StreamChunk::Done { eval_count: parsed.eval_count.unwrap_or(0) as u32 };
                    return ;
                }
                yield StreamChunk::Token(parsed.message.content);
            }
        }
    }
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
    #[allow(dead_code)]
    done: bool,
}

/// Represents a response from the Ollama API stream endpoint.
#[derive(Deserialize)]
struct OllamaStreamResponse {
    message: OllamaMessage,
    done: bool,
    eval_count: Option<usize>,
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
    // TODO : Error test coverage — Consider adding mock-based tests later (requires test harness) or document why real Ollama is required.
    use futures::TryStreamExt;

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

        match client.complete("gemma4", messages).await {
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

    // Test qui appelle stream() et collecte tous les tokens en une String non vide
    #[tokio::test]
    #[ignore = "Ollama needs to be running"]
    async fn collect_stream_tokens() {
        let client = OllamaClient::new("http://localhost:11434");
        let messages = vec![OllamaMessage {
            role: "user".to_string(),
            content: "Hi hello".to_string(),
        }];
        let mut collected_tokens = String::new();
        let mut eval_count = 0u32;
        let model = "gemma4";
        let stream = client.stream(model, messages);

        match stream.try_collect::<Vec<_>>().await {
            Ok(chunks) => {
                for chunk in chunks {
                    match chunk {
                        StreamChunk::Token(token) => collected_tokens.push_str(&token),
                        StreamChunk::Done { eval_count: count } => eval_count = count,
                    }
                }
            }
            Err(e) => panic!("Unexpected Stream error: {:?}", e),
        }
        println!("Réponse complète : {}", collected_tokens);
        assert!(!collected_tokens.trim().is_empty());
        println!("Eval Count : {}", eval_count);
    }
}
