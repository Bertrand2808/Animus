use animus_core::{
    ContentRating, Persona,
    persona::{Message, Role, Summary},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaMessage {
    pub role: String,
    pub content: String,
}

impl OllamaMessage {
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        OllamaMessage {
            role: role.to_string(),
            content: content.into(),
        }
    }

    pub fn role_from_string(role: &str) -> Result<Role, String> {
        match role {
            "user" => Ok(Role::User),
            "assistant" => Ok(Role::Assistant),
            "system" => Ok(Role::System),
            _ => Err(format!("Invalid role: {}", role)),
        }
    }
}

// TODO : Add tests for edge cases: empty messages, single message with summary, message count ≤10.
/*
Risk Assessment
  - CC=6: Moderate complexity (if/match statements for optional summary, first message checks).
  - Coverage=89.3%: Strong test coverage. 3 of 28 lines lack execution (likely edge cases or unused branches).
  - CRAP=6.04: Just above safe zone (≤5). Low risk, but watch for untested branches.
---
What's Not Covered?

  Based on line ranges (31–88):
  - Lines 69–78: History windowing logic (skip/slice calculations) appears fully covered by test 2 (12-message
  scenario).
  - Likely uncovered: Edge cases in skip logic for boundary message counts, or conditional branches in optional
  summary/first_message blocks.
*/
pub fn build_prompt(
    persona: &Persona,
    messages: &[Message],
    summary: Option<&Summary>,
) -> Vec<OllamaMessage> {
    let mut blocks = Vec::new();

    // Bloc 1 : Principal system
    blocks.push(OllamaMessage {
        role: "system".to_string(),
        content: format!(
            "You are {}. \n\nDescription: {}\n\nPersonality: {}\n\nScenario: {}\n\n{}",
            persona.name,
            persona.description,
            persona.personality,
            persona.scenario,
            nsfw_section_if_needed(persona.content_rating)
        ),
    });

    // Bloc 2 : Summary si présent
    if let Some(summary) = summary {
        blocks.push(OllamaMessage {
            role: "system".to_string(),
            content: format!("Summary of earlier conversation:\n\n{}", summary.content),
        });
    }

    // Bloc 3 : premier message si applicable
    if should_include_first_message(messages) {
        blocks.push(OllamaMessage {
            role: "assistant".to_string(),
            content: messages[0].content.clone(),
        });
    }

    // Bloc 4 : historique (derniers 10)
    let hitory_messages = if messages.len() > 1 {
        // Skip premier message si c'était l'assistant (déjà dans le bloc 2)
        let start_index = if should_include_first_message(messages) {
            1
        } else {
            0
        };
        let skip_count = messages.len().saturating_sub(10 + start_index);
        messages
            .iter()
            .skip(start_index)
            .skip(skip_count)
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

    for msg in hitory_messages {
        blocks.push(OllamaMessage {
            role: msg.role.to_string(),
            content: msg.content.clone(),
        });
    }

    blocks
}

// Helper vérifier si on inclut le premier message
fn should_include_first_message(messages: &[Message]) -> bool {
    messages.len() == 1 && messages[0].role == Role::Assistant
}

// Helper section NSFW si rating = Nsfw
fn nsfw_section_if_needed(rating: ContentRating) -> String {
    match rating {
        ContentRating::Nsfw => {
            "NSFW Content Warning: This character may discuss adult content.".to_string()
        }
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use animus_core::{
        content_rating::ContentRating,
        persona::{Message, Persona, Role, Summary},
    };
    use uuid::Uuid;

    fn create_test_persona() -> Persona {
        Persona {
            id: Uuid::now_v7(),
            name: "TestPersona".to_string(),
            description: "Test description".to_string(),
            personality: "Friendly and helpful".to_string(),
            scenario: "Test".to_string(),
            first_message: "Test".to_string(),
            message_example: "Test".to_string(),
            avatar_url: Some("Test".to_string()),
            background_url: Some("Test".to_string()),
            content_rating: ContentRating::Pg,
            model: Some("Test".to_string()),
            raw_card: Some("Test".to_string()),
        }
    }

    fn create_test_message(role: Role, content: &str) -> Message {
        Message {
            id: Uuid::now_v7(),
            conversation_id: Uuid::now_v7(),
            role,
            content: content.to_string(),
            token_count: Some(0),
        }
    }

    // Helper function to create a vector of messages
    fn create_messages(count: usize, start_with_assistant: bool) -> Vec<Message> {
        let mut messages: Vec<Message> = Vec::new();
        for i in 0..count {
            let role = if (i % 2 == 0) == start_with_assistant {
                Role::Assistant
            } else {
                Role::User
            };
            messages.push(create_test_message(role, &format!("Message {}", i + 1)));
        }
        messages
    }

    // Test 1 : nouvelle conversation (1 msg assistant, 0 user)
    #[test]
    fn test_build_prompt_new_conversation() {
        let persona = create_test_persona();
        let messages = vec![create_test_message(
            Role::Assistant,
            "Hello, how are you today ?",
        )];
        let result = build_prompt(&persona, &messages, None);

        assert_eq!(
            result.len(),
            2,
            "Nouvelle conversation -> 2 messages (system + 1er msg)"
        );

        // 1. System principal
        assert_eq!(result[0].role, "system");
        assert!(result[0].content.contains("TestPersona"));
        assert!(result[0].content.contains("Test description"));
        assert!(result[0].content.contains("Friendly and helpful"));

        // 2. Premier message assistant
        assert_eq!(result[1].role, "assistant");
        assert_eq!(result[1].content, "Hello, how are you today ?");
    }

    // Test 2 : conversation existante (ne pas dupliquer le 1er message)
    #[test]
    fn test_build_prompt_existing_conversation() {
        let persona = create_test_persona();
        // 12 messages, commence par un assistant
        let messages = create_messages(12, true);
        let result = build_prompt(&persona, &messages, None);

        // On doit avoir : 1 system + 10 derniers message = 11 messages
        assert_eq!(
            result.len(),
            11,
            "Doit contenir le system + les 10 derniers messages"
        );

        // Premier élément = system
        assert_eq!(result[0].role, "system");

        // Les 10 suivant = les derniers messages de l'input (pas les premiers)
        // On ne doit donc pas avoir le message n°2
        let history = &result[1..];
        // Vérifier que c'est bien les 10 derniers
        assert_eq!(history[0].content, "Message 3");
        assert_eq!(history[9].content, "Message 12");

        // vérifier l'ordre chronologique
        for (i, msg) in history.iter().enumerate() {
            assert_eq!(
                msg.role,
                if i % 2 == 0 {
                    Role::Assistant.to_string()
                } else {
                    Role::User.to_string()
                }
            );
        }
    }

    // Test 3 : summary après system
    #[test]
    fn test_build_prompt_with_summary() {
        let persona = create_test_persona();
        let messages = vec![create_test_message(Role::Assistant, "Hello")];
        let summary = Summary {
            id: Uuid::now_v7(),
            conversation_id: Uuid::now_v7(),
            content: "Résumé de la conversation précédente.".to_string(),
            message_range_start: Uuid::now_v7(),
            message_range_end: Uuid::now_v7(),
        };

        let result = build_prompt(&persona, &messages, Some(&summary));

        // Pour une nouvelle conversation AVEC summary :
        // -> system principal + system summary + first_message assistant
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].role, "system"); // principal
        assert_eq!(result[1].role, "system"); // summary
        assert_eq!(result[2].role, "assistant"); // first message

        // Vérifier le contenu
        assert!(
            result[1]
                .content
                .contains("Summary of earlier conversation:")
        );
        assert!(
            result[1]
                .content
                .contains("Résumé de la conversation précédente.")
        );
    }

    #[test]
    fn build_test_prompt_existing_conversation_with_summary() {
        let persona = create_test_persona();
        let messages = create_messages(8, false); // conversation existante

        let summary = Summary {
            id: Uuid::now_v7(),
            conversation_id: Uuid::now_v7(),
            content: "Ancien résumé.".to_string(),
            message_range_start: Uuid::now_v7(),
            message_range_end: Uuid::now_v7(),
        };

        let result = build_prompt(&persona, &messages, Some(&summary));

        // system + summary + 8 messages = 10
        assert_eq!(result.len(), 10);
        assert_eq!(result[0].role, "system");
        assert_eq!(result[1].role, "system");
        assert_eq!(result[2].role, "user"); // premier du historique (ici user car false)
    }
}
