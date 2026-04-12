use uuid::Uuid;
use crate::ContentRating;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Persona {
  pub id: Uuid,
  pub name: String,
  pub description: String,
  pub personality: String,
  pub scenario: String,
  pub first_message: String,
  pub message_example: String,
  pub avatar_url: Option<String>,
  pub background_url: Option<String>,
  pub content_rating: ContentRating,
  pub model: Option<String>,
  pub raw_card: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
  User,
  Assistant,
  System,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
  pub id: Uuid,
  pub conversation_id: Uuid,
  pub role: Role,
  pub content: String,
  pub token_count: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Conversation {
  pub id: Uuid,
  pub persona_id: Uuid,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Summary {
  pub id: Uuid,
  pub conversation_id: Uuid,
  pub content: String,
  pub message_range_start: Uuid,
  pub message_range_end: Uuid,
}
