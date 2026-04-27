use core::fmt;

use crate::{CharacterCardV2, ContentRating};
use uuid::Uuid;

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
    pub raw_card: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum CardImportError {
    #[error("card spec must be 'chara_card_v2', got '{0}'")]
    InvalidSpec(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl TryFrom<CharacterCardV2> for Persona {
    type Error = CardImportError;

    fn try_from(card: CharacterCardV2) -> Result<Self, Self::Error> {
        if card.spec != "chara_card_v2" {
            return Err(CardImportError::InvalidSpec(card.spec));
        }
        let raw_card = serde_json::to_string(&card)?;
        let content_rating = card.data.content_rating();
        Ok(Persona {
            id: Uuid::now_v7(),
            name: card.data.name,
            description: card.data.description,
            personality: card.data.personality,
            scenario: card.data.scenario,
            first_message: card.data.first_mes,
            message_example: card.data.mes_example,
            avatar_url: None,
            background_url: None,
            content_rating,
            model: None,
            raw_card: Some(raw_card),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::User => write!(f, "user"),
            Role::Assistant => write!(f, "assistant"),
            Role::System => write!(f, "system"),
        }
    }
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
    pub created_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CharacterCardV2;

    fn valid_card_json() -> &'static str {
        r#"{
            "spec": "chara_card_v2",
            "spec_version": "2.0",
            "data": {
                "name": "Aria",
                "description": "A helpful AI",
                "personality": "Calm",
                "scenario": "Library",
                "first_mes": "Hello!",
                "mes_example": ""
            }
        }"#
    }

    #[test]
    fn try_from_valid_card() {
        let card: CharacterCardV2 = serde_json::from_str(valid_card_json()).unwrap();
        let persona = Persona::try_from(card).unwrap();
        assert_eq!(persona.name, "Aria");
        assert_eq!(persona.first_message, "Hello!");
        assert_eq!(persona.content_rating, ContentRating::Pg);
        assert!(persona.raw_card.is_some());
    }

    #[test]
    fn try_from_invalid_spec_returns_error() {
        let json = r#"{
            "spec": "chara_card_v1",
            "spec_version": "1.0",
            "data": { "name": "Old" }
        }"#;
        let card: CharacterCardV2 = serde_json::from_str(json).unwrap();
        let err = Persona::try_from(card).unwrap_err();
        assert!(matches!(err, CardImportError::InvalidSpec(_)));
    }

    #[test]
    fn try_from_nsfw_card() {
        let json = r#"{
            "spec": "chara_card_v2",
            "spec_version": "2.0",
            "data": {
                "name": "Aria",
                "extensions": { "content_rating": "nsfw" }
            }
        }"#;
        let card: CharacterCardV2 = serde_json::from_str(json).unwrap();
        let persona = Persona::try_from(card).unwrap();
        assert_eq!(persona.content_rating, ContentRating::Nsfw);
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Summary {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub content: String,
    pub message_range_start: Uuid,
    pub message_range_end: Uuid,
}
