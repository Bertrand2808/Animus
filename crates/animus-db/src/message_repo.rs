use animus_core::persona::{Message, Role};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Clone)]
pub struct MessageRepo {
    pool: SqlitePool,
}

impl MessageRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, message: &Message) -> Result<(), sqlx::Error> {
        let msg_id = message.id.to_string();
        let conv_id = message.conversation_id.to_string();
        let role = role_to_str(&message.role);
        let content = &message.content;
        let token_count = message.token_count;

        sqlx::query!(
            r#"
            INSERT INTO messages (id, conversation_id, role, content, token_count, created_at)
            VALUES (?, ?, ?, ?, ?, unixepoch())
            "#,
            msg_id,
            conv_id,
            role,
            content,
            token_count,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_last_n(
        &self,
        conversation_id: Uuid,
        n: i64,
    ) -> Result<Vec<Message>, sqlx::Error> {
        let id_str = conversation_id.to_string();
        let rows = sqlx::query!(
            r#"
            SELECT id AS "id!", conversation_id AS "conversation_id!", role AS "role!", content AS "content!", token_count
            FROM (
                SELECT id, conversation_id, role, content, token_count
                FROM messages
                WHERE conversation_id = ?
                ORDER BY id DESC
                LIMIT ?
            )
            ORDER BY id ASC
            "#,
            id_str,
            n
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| {
                Ok(Message {
                    id: r.id.parse().map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                    conversation_id: r
                        .conversation_id
                        .parse()
                        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                    role: str_to_role(&r.role).map_err(sqlx::Error::Decode)?,
                    content: r.content,
                    token_count: r.token_count,
                })
            })
            .collect()
    }
}

fn role_to_str(role: &Role) -> &'static str {
    match role {
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::System => "system",
    }
}

fn str_to_role(s: &str) -> Result<Role, Box<dyn std::error::Error + Send + Sync>> {
    match s {
        "user" => Ok(Role::User),
        "assistant" => Ok(Role::Assistant),
        "system" => Ok(Role::System),
        other => Err(format!("unknown role: {other}").into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ContentRating, Persona, conversation_repo::ConversationRepo, persona_repo::PersonaRepo,
    };
    use animus_core::persona::Conversation;
    use sqlx::SqlitePool;

    async fn seed_conversation(pool: &SqlitePool) -> Uuid {
        let persona = Persona {
            id: Uuid::now_v7(),
            name: format!("test-{}", Uuid::now_v7()),
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
        };
        PersonaRepo::new(pool.clone())
            .insert(&persona)
            .await
            .unwrap();

        let conv = Conversation {
            id: Uuid::now_v7(),
            persona_id: persona.id,
            created_at: 0,
        };
        ConversationRepo::new(pool.clone())
            .insert(&conv)
            .await
            .unwrap();
        conv.id
    }

    fn make_message(conversation_id: Uuid, role: Role) -> Message {
        Message {
            id: Uuid::now_v7(),
            conversation_id,
            role,
            content: "hello".to_string(),
            token_count: Some(1),
        }
    }

    #[sqlx::test]
    async fn insert_and_find_last_10(pool: SqlitePool) {
        let conv_id = seed_conversation(&pool).await;
        let repo = MessageRepo::new(pool);

        for _ in 0..15 {
            repo.insert(&make_message(conv_id, Role::User))
                .await
                .unwrap();
        }

        let messages = repo.find_last_n(conv_id, 10).await.unwrap();
        assert_eq!(messages.len(), 10);

        // chronological order: each id >= previous
        for w in messages.windows(2) {
            assert!(w[0].id <= w[1].id);
        }
    }

    #[sqlx::test]
    async fn find_last_n_empty_conv(pool: SqlitePool) {
        let conv_id = seed_conversation(&pool).await;
        let repo = MessageRepo::new(pool);
        let messages = repo.find_last_n(conv_id, 10).await.unwrap();
        assert!(messages.is_empty());
    }

    #[sqlx::test]
    async fn roles_roundtrip(pool: SqlitePool) {
        let conv_id = seed_conversation(&pool).await;
        let repo = MessageRepo::new(pool);

        for role in [Role::User, Role::Assistant, Role::System] {
            repo.insert(&make_message(conv_id, role)).await.unwrap();
        }

        let messages = repo.find_last_n(conv_id, 10).await.unwrap();
        assert_eq!(messages.len(), 3);

        let roles: Vec<&Role> = messages.iter().map(|m| &m.role).collect();
        assert!(roles.contains(&&Role::User));
        assert!(roles.contains(&&Role::Assistant));
        assert!(roles.contains(&&Role::System));
    }
}
