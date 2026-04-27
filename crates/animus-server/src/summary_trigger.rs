use animus_core::persona::Summary;
use animus_llm::OllamaMessage;
use uuid::Uuid;

use crate::state::AppState;

pub async fn evaluate_summary_trigger(conv_id: Uuid, state: AppState) {
    let latest_summary = match state.summaries.find_latest(conv_id).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(conversation_id = %conv_id, "failed to fetch latest summary: {:?}", e);
            return;
        }
    };

    let uncovered = match &latest_summary {
        None => match state.messages.find_last_n(conv_id, i64::MAX).await {
            Ok(msgs) => msgs,
            Err(e) => {
                tracing::warn!(conversation_id = %conv_id, "failed to fetch all messages: {:?}", e);
                return;
            }
        },
        Some(summary) => {
            match state
                .messages
                .find_after(conv_id, summary.message_range_end)
                .await
            {
                Ok(msgs) => msgs,
                Err(e) => {
                    tracing::warn!(conversation_id = %conv_id, "failed to fetch messages after summary: {:?}", e);
                    return;
                }
            }
        }
    };

    if uncovered.len() < 20 {
        return;
    }

    let (Some(first_msg), Some(last_msg)) = (uncovered.first(), uncovered.last()) else {
        return;
    };
    let range_start = first_msg.id;
    let range_end = last_msg.id;

    let mut prompt = String::with_capacity(2048);
    prompt.push_str(
        "You are summarizing a roleplay conversation.\n\
        Respond in 3-5 sentences maximum. Focus on: character names, \
        key events, current emotional state, unresolved tensions.",
    );
    if let Some(prev) = &latest_summary {
        prompt.push_str("\n\nPrevious summary:\n");
        prompt.push_str(&prev.content);
    }
    prompt.push_str("\n\nMessages to summarize:\n");
    for msg in &uncovered {
        prompt.push_str(&msg.role.to_string().to_uppercase());
        prompt.push_str(": ");
        prompt.push_str(&msg.content);
        prompt.push('\n');
    }

    let content = match state
        .ollama
        .complete(
            &state.model_name,
            vec![OllamaMessage {
                role: "user".to_owned(),
                content: prompt,
            }],
        )
        .await
    {
        Ok(text) => text,
        Err(e) => {
            tracing::warn!(conversation_id = %conv_id, "ollama failed during summary generation: {:?}", e);
            return;
        }
    };

    let summary = Summary {
        id: Uuid::now_v7(),
        conversation_id: conv_id,
        content,
        message_range_start: range_start,
        message_range_end: range_end,
    };

    if let Err(e) = state.summaries.insert(&summary).await {
        tracing::warn!(conversation_id = %conv_id, "failed to insert summary: {:?}", e);
        return;
    }

    tracing::info!(conversation_id = %conv_id, "summary generated");
}
