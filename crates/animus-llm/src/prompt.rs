use std::borrow::Cow;

use animus_core::{
    persona::{Message, Role, Summary},
    Persona,
};
use serde::{Deserialize, Serialize};

const DEFAULT_TEMPLATE: &str = "Write as {{char}} in a character-driven roleplay scene.\n\
Use vivid physical actions in *asterisks* and spoken dialogue in \"quotes\".\n\
Stay in {{char}}'s perspective and preserve {{char}}'s personality, goals, speech patterns, and boundaries.\n\
Drive the scene proactively through concrete actions, choices, tension, and sensory detail.\n\
Keep the response under {{response_length_limit}} characters.";

const NSFW_TEMPLATE: &str = "Write as {{char}} in a mature, intimate roleplay scene.\n\
Use vivid physical actions in *asterisks* and spoken dialogue in \"quotes\".\n\
Stay in {{char}}'s perspective and preserve {{char}}'s personality, desires, goals, speech patterns, and boundaries.\n\
Build tension through pacing, sensory detail, consent-aware reactions, and proactive character choices.\n\
Keep the response under {{response_length_limit}} characters.";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PromptTraceFields {
    message_count: usize,
    has_summary: bool,
    has_model_instructions: bool,
    has_appearance: bool,
    has_speech_style: bool,
    has_character_goals: bool,
    has_post_history_instructions: bool,
    instruction_template: String,
    response_length_limit: i64,
}

impl PromptTraceFields {
    fn from_inputs(persona: &Persona, messages: &[Message], has_summary: bool) -> Self {
        Self {
            message_count: messages.len(),
            has_summary,
            has_model_instructions: !persona.model_instructions.trim().is_empty(),
            has_appearance: !persona.appearance.trim().is_empty(),
            has_speech_style: !persona.speech_style.trim().is_empty(),
            has_character_goals: !persona.character_goals.trim().is_empty(),
            has_post_history_instructions: !persona.post_history_instructions.trim().is_empty(),
            instruction_template: persona.instruction_template.clone(),
            response_length_limit: persona.response_length_limit,
        }
    }
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

/// Replaces all known prompt placeholders with their resolved values.
pub fn resolve_placeholders(text: &str, char_name: &str, user_name: &str, limit: i64) -> String {
    let words_approx = limit / 4;
    text.replace("{{char}}", char_name)
        .replace("{{user}}", user_name)
        .replace("{{response_length_limit}}", &limit.to_string())
        .replace(
            "{{response_length_example}}",
            &format!("about {} words", words_approx),
        )
}

/// Returns the raw (unresolved) template text for the persona.
///
/// - `"default"` → DEFAULT_TEMPLATE
/// - `"nsfw"` → NSFW_TEMPLATE
/// - `"custom"` or any unknown value → persona's `model_instructions`
///
/// Unknown values map to `model_instructions` so that stored custom text is never silently
/// dropped when an unrecognised template name reaches the prompt builder.
fn select_template(persona: &Persona) -> Cow<'_, str> {
    match persona.instruction_template.as_str() {
        "default" => Cow::Borrowed(DEFAULT_TEMPLATE),
        "nsfw" => Cow::Borrowed(NSFW_TEMPLATE),
        _ => Cow::Borrowed(persona.model_instructions.as_str()),
    }
}

/// Builds the main structured system block from persona fields.
fn build_system_block(persona: &Persona, user_name: &str) -> String {
    let char_name = &persona.name;
    let limit = persona.response_length_limit;
    let mut sections: Vec<String> = Vec::with_capacity(8);

    // # Role
    sections.push(format!("# Role\nYou are {char_name}."));

    // # Model Instructions — skip if template resolves to empty (e.g. custom + empty model_instructions)
    let template = select_template(persona);
    if !template.trim().is_empty() {
        let resolved = resolve_placeholders(&template, char_name, user_name, limit);
        sections.push(format!("# Model Instructions\n{resolved}"));
    }

    // Helper: resolve + return Some only when non-empty
    let resolve = |text: &str| -> Option<String> {
        let s = text.trim();
        if s.is_empty() {
            None
        } else {
            Some(resolve_placeholders(s, char_name, user_name, limit))
        }
    };

    // # Character — only include non-empty sub-fields
    {
        let mut parts: Vec<String> = Vec::new();
        if let Some(v) = resolve(&persona.appearance) {
            parts.push(format!("Appearance:\n{v}"));
        }
        if let Some(v) = resolve(&persona.description) {
            parts.push(format!("Description:\n{v}"));
        }
        if let Some(v) = resolve(&persona.personality) {
            parts.push(format!("Personality:\n{v}"));
        }
        if let Some(v) = resolve(&persona.speech_style) {
            parts.push(format!("Speech Style:\n{v}"));
        }
        if !parts.is_empty() {
            sections.push(format!("# Character\n{}", parts.join("\n\n")));
        }
    }

    // # Scenario
    if let Some(v) = resolve(&persona.scenario) {
        sections.push(format!("# Scenario\n{v}"));
    }

    // # Character Goals
    if let Some(v) = resolve(&persona.character_goals) {
        sections.push(format!("# Character Goals\n{v}"));
    }

    // # Style Examples
    if let Some(v) = resolve(&persona.message_example) {
        sections.push(format!(
            "# Style Examples\nUse these as style references, not current conversation events.\n{v}"
        ));
    }

    // # Response Contract — always present
    let words_approx = limit / 4;
    sections.push(format!(
        "# Response Contract\nKeep the response under {limit} characters (about {words_approx} words)."
    ));

    sections.join("\n\n")
}

fn should_include_first_message(messages: &[Message]) -> bool {
    messages.len() == 1 && messages[0].role == Role::Assistant
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
    user_name: &str,
) -> Vec<OllamaMessage> {
    let trace = PromptTraceFields::from_inputs(persona, messages, summary.is_some());
    tracing::debug!(
        target: "ollama_prompt",
        persona_id = %persona.id,
        persona_name = %persona.name,
        message_count = trace.message_count,
        has_summary = trace.has_summary,
        has_model_instructions = trace.has_model_instructions,
        has_appearance = trace.has_appearance,
        has_speech_style = trace.has_speech_style,
        has_character_goals = trace.has_character_goals,
        has_post_history_instructions = trace.has_post_history_instructions,
        response_length_limit = trace.response_length_limit,
        temperature = persona.temperature,
        repeat_penalty = persona.repeat_penalty,
        instruction_template = %trace.instruction_template,
        "building prompt for ollama"
    );

    let mut blocks = Vec::new();

    // Block 1: main system message with structured sections
    blocks.push(OllamaMessage {
        role: "system".to_string(),
        content: build_system_block(persona, user_name),
    });

    // Block 2: summary
    if let Some(summary) = summary {
        blocks.push(OllamaMessage {
            role: "system".to_string(),
            content: format!("Summary of earlier conversation:\n\n{}", summary.content),
        });
    }

    // Block 3: first message (new conversation — single assistant message)
    if should_include_first_message(messages) {
        blocks.push(OllamaMessage {
            role: "assistant".to_string(),
            content: messages[0].content.clone(),
        });
    }

    // Block 4: conversation history (last 10, skipping first message if in block 3)
    let history_messages = if messages.len() > 1 {
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

    for msg in history_messages {
        blocks.push(OllamaMessage {
            role: msg.role.to_string(),
            content: msg.content.clone(),
        });
    }

    // Block 5: post-history instructions (optional — injected after history)
    if !persona.post_history_instructions.trim().is_empty() {
        let resolved = resolve_placeholders(
            &persona.post_history_instructions,
            &persona.name,
            user_name,
            persona.response_length_limit,
        );
        blocks.push(OllamaMessage {
            role: "system".to_string(),
            content: resolved,
        });
    }

    tracing::debug!(
        target: "ollama_prompt",
        persona_id = %persona.id,
        block_count = blocks.len(),
        first_message_included = should_include_first_message(messages),
        has_post_history = !persona.post_history_instructions.trim().is_empty(),
        "prompt built for ollama"
    );

    blocks
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
            model_instructions: String::new(),
            appearance: String::new(),
            speech_style: String::new(),
            character_goals: String::new(),
            post_history_instructions: String::new(),
            response_length_limit: 1200,
            temperature: 0.65,
            repeat_penalty: 1.12,
            instruction_template: "default".to_owned(),
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

    // --- PromptTraceFields ---

    #[test]
    fn prompt_trace_fields_report_structured_field_presence() {
        let mut persona = create_test_persona();
        persona.model_instructions = "Stay in character".to_owned();
        persona.speech_style = "Short sentences".to_owned();
        persona.response_length_limit = 900;
        persona.instruction_template = "cinematic".to_owned();
        let messages = create_messages(3, false);

        let fields = PromptTraceFields::from_inputs(&persona, &messages, true);

        assert_eq!(
            fields,
            PromptTraceFields {
                message_count: 3,
                has_summary: true,
                has_model_instructions: true,
                has_appearance: false,
                has_speech_style: true,
                has_character_goals: false,
                has_post_history_instructions: false,
                instruction_template: "cinematic".to_owned(),
                response_length_limit: 900,
            }
        );
    }

    // --- resolve_placeholders ---

    #[test]
    fn resolve_placeholders_replaces_all_known_tokens() {
        let result = resolve_placeholders(
            "Hello {{char}}, I am {{user}}. Limit: {{response_length_limit}}. Example: {{response_length_example}}.",
            "Aria",
            "Bertrand",
            1200,
        );
        assert_eq!(
            result,
            "Hello Aria, I am Bertrand. Limit: 1200. Example: about 300 words."
        );
    }

    #[test]
    fn resolve_placeholders_response_length_example_is_human_readable() {
        let result = resolve_placeholders("{{response_length_example}}", "X", "Y", 800);
        assert_eq!(result, "about 200 words");
    }

    // --- select_template ---

    #[test]
    fn select_template_default_returns_default_template() {
        let mut persona = create_test_persona();
        persona.instruction_template = "default".to_owned();
        persona.content_rating = ContentRating::Nsfw; // content_rating must NOT affect selection
        assert_eq!(select_template(&persona).as_ref(), DEFAULT_TEMPLATE);
    }

    #[test]
    fn select_template_nsfw_returns_nsfw_template() {
        let mut persona = create_test_persona();
        persona.instruction_template = "nsfw".to_owned();
        assert_eq!(select_template(&persona).as_ref(), NSFW_TEMPLATE);
    }

    #[test]
    fn select_template_custom_returns_model_instructions() {
        let mut persona = create_test_persona();
        persona.instruction_template = "custom".to_owned();
        persona.model_instructions = "My custom instructions.".to_owned();
        assert_eq!(select_template(&persona).as_ref(), "My custom instructions.");
    }

    #[test]
    fn select_template_unknown_value_falls_back_to_model_instructions() {
        let mut persona = create_test_persona();
        persona.instruction_template = "cinematic".to_owned();
        persona.model_instructions = "Cinematic custom instructions.".to_owned();
        assert_eq!(
            select_template(&persona).as_ref(),
            "Cinematic custom instructions.",
            "unknown instruction_template must use model_instructions, not silently drop them"
        );
    }

    #[test]
    fn custom_template_not_overridden_by_nsfw_persona() {
        let mut persona = create_test_persona();
        persona.content_rating = ContentRating::Nsfw;
        persona.instruction_template = "custom".to_owned();
        persona.model_instructions = "Custom style only.".to_owned();

        let system = build_system_block(&persona, "User");

        assert!(
            system.contains("Custom style only."),
            "custom model_instructions should be used"
        );
        assert!(
            !system.contains("mature, intimate"),
            "NSFW template must not be injected when instruction_template=custom"
        );
    }

    // --- build_system_block ---

    #[test]
    fn system_block_has_required_sections() {
        let persona = create_test_persona();
        let block = build_system_block(&persona, "User");

        assert!(block.contains("# Role"), "missing # Role");
        assert!(block.contains("# Model Instructions"), "missing # Model Instructions");
        assert!(block.contains("# Response Contract"), "missing # Response Contract");
    }

    #[test]
    fn system_block_omits_empty_optional_sections() {
        let mut persona = create_test_persona();
        // All optional fields empty
        persona.appearance = String::new();
        persona.description = String::new();
        persona.personality = String::new();
        persona.speech_style = String::new();
        persona.scenario = String::new();
        persona.character_goals = String::new();
        persona.message_example = String::new();

        let block = build_system_block(&persona, "User");

        assert!(!block.contains("# Character"), "# Character should be omitted when all sub-fields empty");
        assert!(!block.contains("# Scenario"), "# Scenario should be omitted when empty");
        assert!(!block.contains("# Character Goals"), "# Character Goals should be omitted when empty");
        assert!(!block.contains("# Style Examples"), "# Style Examples should be omitted when empty");
    }

    #[test]
    fn system_block_includes_optional_sections_when_populated() {
        let mut persona = create_test_persona();
        persona.appearance = "Tall, dark hair.".to_owned();
        persona.speech_style = "Terse.".to_owned();
        persona.scenario = "A rainy city.".to_owned();
        persona.character_goals = "Protect the guild.".to_owned();
        persona.message_example = "Example reply.".to_owned();

        let block = build_system_block(&persona, "User");

        assert!(block.contains("# Character"));
        assert!(block.contains("Appearance:"));
        assert!(block.contains("Speech Style:"));
        assert!(block.contains("# Scenario"));
        assert!(block.contains("# Character Goals"));
        assert!(block.contains("# Style Examples"));
        assert!(block.contains("Use these as style references"));
    }

    #[test]
    fn system_block_no_unresolved_placeholders() {
        let mut persona = create_test_persona();
        persona.description = "{{char}} is mysterious.".to_owned();
        persona.character_goals = "Please {{user}} always.".to_owned();

        let block = build_system_block(&persona, "Alice");

        assert!(
            !block.contains("{{"),
            "unresolved placeholder found in system block: {block}"
        );
    }

    #[test]
    fn system_block_resolves_char_and_user_placeholders() {
        let mut persona = create_test_persona();
        persona.name = "Aria".to_owned();

        let block = build_system_block(&persona, "Bertrand");

        assert!(block.contains("You are Aria."));
        assert!(block.contains("Aria")); // from default template
        assert!(!block.contains("{{char}}"));
        assert!(!block.contains("{{user}}"));
    }

    #[test]
    fn system_block_response_contract_uses_limit() {
        let mut persona = create_test_persona();
        persona.response_length_limit = 800;

        let block = build_system_block(&persona, "User");

        assert!(block.contains("# Response Contract"));
        assert!(block.contains("800 characters"));
        assert!(block.contains("about 200 words"));
    }

    // --- build_prompt (existing tests updated for user_name) ---

    #[test]
    fn test_build_prompt_new_conversation() {
        let persona = create_test_persona();
        let messages = vec![create_test_message(
            Role::Assistant,
            "Hello, how are you today ?",
        )];
        let result = build_prompt(&persona, &messages, None, "User");

        assert_eq!(result.len(), 2, "new conversation -> system + first message");

        assert_eq!(result[0].role, "system");
        assert!(result[0].content.contains("# Role"));
        assert!(result[0].content.contains("TestPersona"));

        assert_eq!(result[1].role, "assistant");
        assert_eq!(result[1].content, "Hello, how are you today ?");
    }

    #[test]
    fn test_build_prompt_existing_conversation() {
        let persona = create_test_persona();
        let messages = create_messages(12, true);
        let result = build_prompt(&persona, &messages, None, "User");

        assert_eq!(result.len(), 11, "system + 10 last messages");

        assert_eq!(result[0].role, "system");

        let history = &result[1..];
        assert_eq!(history[0].content, "Message 3");
        assert_eq!(history[9].content, "Message 12");

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
            created_at: 0,
        };

        let result = build_prompt(&persona, &messages, Some(&summary), "User");

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].role, "system"); // main system block
        assert_eq!(result[1].role, "system"); // summary
        assert_eq!(result[2].role, "assistant"); // first message

        assert!(result[1].content.contains("Summary of earlier conversation:"));
        assert!(result[1].content.contains("Résumé de la conversation précédente."));
    }

    #[test]
    fn build_test_prompt_existing_conversation_with_summary() {
        let persona = create_test_persona();
        let messages = create_messages(8, false);

        let summary = Summary {
            id: Uuid::now_v7(),
            conversation_id: Uuid::now_v7(),
            content: "Ancien résumé.".to_string(),
            message_range_start: Uuid::now_v7(),
            message_range_end: Uuid::now_v7(),
            created_at: 0,
        };

        let result = build_prompt(&persona, &messages, Some(&summary), "User");

        assert_eq!(result.len(), 10); // system + summary + 8 messages
        assert_eq!(result[0].role, "system");
        assert_eq!(result[1].role, "system");
        assert_eq!(result[2].role, "user");
    }

    // --- post-history instructions ---

    #[test]
    fn build_prompt_appends_post_history_block_when_set() {
        let mut persona = create_test_persona();
        persona.post_history_instructions = "Stay focused on the scene, {{char}}.".to_owned();
        let messages = create_messages(4, false);

        let result = build_prompt(&persona, &messages, None, "User");

        let last = result.last().expect("prompt must not be empty");
        assert_eq!(last.role, "system");
        assert!(
            last.content.contains("Stay focused on the scene, TestPersona."),
            "post_history_instructions should be resolved and appended: {:?}",
            last.content
        );
    }

    #[test]
    fn build_prompt_no_post_history_block_when_empty() {
        let persona = create_test_persona(); // post_history_instructions is empty
        let messages = create_messages(4, false);

        let result = build_prompt(&persona, &messages, None, "User");

        // Last block must be a user/assistant message, not a system post-history block
        let last = result.last().unwrap();
        assert_ne!(
            last.role, "system",
            "no extra system block expected when post_history_instructions is empty"
        );
    }

    // --- no unresolved placeholders end-to-end ---

    #[test]
    fn build_prompt_leaves_no_unresolved_placeholders() {
        let mut persona = create_test_persona();
        persona.description = "{{char}} is a noble warrior.".to_owned();
        persona.character_goals = "Serve {{user}} well.".to_owned();
        persona.post_history_instructions = "Remember: {{char}} never retreats.".to_owned();

        let messages = create_messages(4, false);
        let result = build_prompt(&persona, &messages, None, "Bertrand");

        for block in &result {
            assert!(
                !block.content.contains("{{"),
                "unresolved placeholder in block role={}: {:?}",
                block.role,
                block.content
            );
        }
    }

    // --- NSFW template ---

    #[test]
    fn build_prompt_uses_nsfw_template_when_instruction_template_is_nsfw() {
        let mut persona = create_test_persona();
        persona.instruction_template = "nsfw".to_owned();
        let messages = create_messages(2, false);

        let result = build_prompt(&persona, &messages, None, "User");

        let system = &result[0].content;
        assert!(
            system.contains("mature, intimate"),
            "NSFW template not found in system block: {system}"
        );
        assert!(
            !system.contains("character-driven roleplay"),
            "default template must not appear when nsfw is selected"
        );
    }
}
