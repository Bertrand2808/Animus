use crate::{ContentRating, Conversation, Persona};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Clone)]
pub struct ConversationRepo {
    pool: SqlitePool,
}

impl ConversationRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, conv: &Conversation) -> Result<(), sqlx::Error> {
        let id = conv.id.to_string();
        let persona_id = conv.persona_id.to_string();
        sqlx::query!(
            r#"
      INSERT INTO conversations (id, persona_id, created_at, updated_at)
      VALUES (?, ?, unixepoch(), unixepoch())
      "#,
            id,
            persona_id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Fetch a conversation and its persona in a single JOIN query.
    pub async fn find_by_id_with_persona(
        &self,
        id: Uuid,
    ) -> Result<Option<(Conversation, Persona)>, sqlx::Error> {
        let id_str = id.to_string();
        let row = sqlx::query!(
            r#"
            SELECT
                c.id          AS "conv_id!",
                c.persona_id  AS "conv_persona_id!",
                c.created_at  AS "conv_created_at!",
                p.id          AS "p_id!",
                p.name        AS "p_name!",
                p.description AS "p_description!",
                p.personality AS "p_personality!",
                p.scenario    AS "p_scenario!",
                p.first_message  AS "p_first_message!",
                p.message_example AS "p_message_example!",
                p.avatar_url,
                p.background_url,
                p.content_rating AS "p_content_rating!",
                p.model,
                p.raw_card,
                p.model_instructions AS "p_model_instructions!",
                p.appearance AS "p_appearance!",
                p.speech_style AS "p_speech_style!",
                p.character_goals AS "p_character_goals!",
                p.post_history_instructions AS "p_post_history_instructions!",
                p.response_length_limit AS "p_response_length_limit!",
                p.temperature AS "p_temperature!",
                p.repeat_penalty AS "p_repeat_penalty!",
                p.instruction_template AS "p_instruction_template!"
            FROM conversations c
            INNER JOIN personas p ON c.persona_id = p.id
            WHERE c.id = ?
            "#,
            id_str
        )
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| {
            let conv = Conversation {
                id: r
                    .conv_id
                    .parse()
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                persona_id: r
                    .conv_persona_id
                    .parse()
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                created_at: r.conv_created_at,
            };
            let persona = Persona {
                id: r
                    .p_id
                    .parse()
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                name: r.p_name,
                description: r.p_description,
                personality: r.p_personality,
                scenario: r.p_scenario,
                first_message: r.p_first_message,
                message_example: r.p_message_example,
                avatar_url: r.avatar_url,
                background_url: r.background_url,
                content_rating: r
                    .p_content_rating
                    .parse::<ContentRating>()
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                model: r.model,
                raw_card: r.raw_card,
                model_instructions: r.p_model_instructions,
                appearance: r.p_appearance,
                speech_style: r.p_speech_style,
                character_goals: r.p_character_goals,
                post_history_instructions: r.p_post_history_instructions,
                response_length_limit: r.p_response_length_limit,
                temperature: r.p_temperature,
                repeat_penalty: r.p_repeat_penalty,
                instruction_template: r.p_instruction_template,
            };
            Ok((conv, persona))
        })
        .transpose()
    }

    pub async fn find_latest_by_persona_id(
        &self,
        persona_id: Uuid,
    ) -> Result<Option<Conversation>, sqlx::Error> {
        let persona_id_str = persona_id.to_string();
        let row = sqlx::query!(
            r#"
            SELECT
                id         AS "id!",
                persona_id AS "persona_id!",
                created_at AS "created_at!"
            FROM conversations
            WHERE persona_id = ?
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            persona_id_str
        )
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| {
            Ok(Conversation {
                id: r.id.parse().map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                persona_id: r
                    .persona_id
                    .parse()
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                created_at: r.created_at,
            })
        })
        .transpose()
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Conversation>, sqlx::Error> {
        let id_string = id.to_string();
        let row = sqlx::query!(
            r#"SELECT id, persona_id, created_at FROM conversations WHERE id = ?"#,
            id_string
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Conversation {
            id: Uuid::parse_str(r.id.as_deref().expect("null id in DB"))
                .expect("invalid UUID in DB"),
            persona_id: Uuid::parse_str(&r.persona_id).expect("invalid UUID in DB"),
            created_at: r.created_at,
        }))
    }
}

// Tests
#[cfg(test)]
mod tests {

    use super::*;
    use crate::{ContentRating, Persona, persona_repo::PersonaRepo};
    use sqlx::SqlitePool;
    use uuid::Uuid;

    /// Helper insert_test_persona creates a real persona in the test DB before inserting the
    /// conversation. insert_unknown_persona_id stays untouched — it still tests FK violation
    /// with a random UUID.
    async fn insert_test_persona(pool: &SqlitePool) -> Uuid {
        use animus_core::persona::{
            DEFAULT_INSTRUCTION_TEMPLATE, DEFAULT_REPEAT_PENALTY, DEFAULT_RESPONSE_LENGTH_LIMIT,
            DEFAULT_TEMPERATURE,
        };

        let persona = Persona {
            id: Uuid::now_v7(),
            name: "test".to_string(),
            description: String::new(),
            personality: String::new(),
            scenario: String::new(),
            first_message: String::new(),
            message_example: String::new(),
            avatar_url: None,
            background_url: None,
            content_rating: ContentRating::Pg,
            model: None,
            raw_card: Some("{}".to_string()),
            model_instructions: String::new(),
            appearance: String::new(),
            speech_style: String::new(),
            character_goals: String::new(),
            post_history_instructions: String::new(),
            response_length_limit: DEFAULT_RESPONSE_LENGTH_LIMIT,
            temperature: DEFAULT_TEMPERATURE,
            repeat_penalty: DEFAULT_REPEAT_PENALTY,
            instruction_template: DEFAULT_INSTRUCTION_TEMPLATE.to_owned(),
        };
        PersonaRepo::new(pool.clone())
            .insert(&persona)
            .await
            .unwrap();
        persona.id
    }

    #[sqlx::test]
    async fn insert_and_find_by_id(pool: SqlitePool) {
        let persona_id = insert_test_persona(&pool).await;
        let conv = Conversation {
            id: Uuid::now_v7(),
            persona_id,
            created_at: 0,
        };

        let repository = ConversationRepo::new(pool);
        repository.insert(&conv).await.unwrap();

        // récupère
        let found = repository.find_by_id(conv.id).await.unwrap();

        // vérifie persona_id
        assert_eq!(found.unwrap().persona_id, conv.persona_id);
    }

    #[sqlx::test]
    async fn find_by_id_not_found(pool: SqlitePool) {
        let repository = ConversationRepo::new(pool);
        let found = repository.find_by_id(Uuid::now_v7()).await.unwrap();
        assert!(found.is_none());
    }

    #[sqlx::test]
    async fn insert_unknown_persona_id(pool: SqlitePool) {
        // FK violation
        let conv = Conversation {
            id: Uuid::now_v7(),
            persona_id: Uuid::now_v7(),
            created_at: 0,
        };

        let repository = ConversationRepo::new(pool);
        let result = repository.insert(&conv).await;
        assert!(result.is_err());
    }
}
