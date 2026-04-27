use animus_core::persona::Summary;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Clone)]
pub struct SummaryRepo {
    pool: SqlitePool,
}

impl SummaryRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn find_latest(&self, conversation_id: Uuid) -> Result<Option<Summary>, sqlx::Error> {
        let id_str = conversation_id.to_string();
        let row = sqlx::query!(
            r#"
            SELECT
                id                  AS "id!",
                conversation_id     AS "conversation_id!",
                content             AS "content!",
                message_range_start AS "message_range_start!",
                message_range_end   AS "message_range_end!"
            FROM summaries
            WHERE conversation_id = ?
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            id_str
        )
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| {
            Ok(Summary {
                id: r.id.parse().map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                conversation_id: r
                    .conversation_id
                    .parse()
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                content: r.content,
                message_range_start: r
                    .message_range_start
                    .parse()
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                message_range_end: r
                    .message_range_end
                    .parse()
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
            })
        })
        .transpose()
    }

    pub async fn insert(&self, summary: &Summary) -> Result<(), sqlx::Error> {
        let summary_id = summary.id.to_string();
        let conversation_id = summary.conversation_id.to_string();
        let message_range_start = summary.message_range_start.to_string();
        let message_range_end = summary.message_range_end.to_string();

        sqlx::query!(
            r#"
            INSERT INTO summaries (id, conversation_id, content, message_range_start, message_range_end, created_at)
            VALUES (?, ?, ?, ?, ?, unixepoch())
            "#,
            summary_id,
            conversation_id,
            summary.content,
            message_range_start,
            message_range_end,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

#[cfg(test)]
mod tests {
    use crate::persona_repo::PersonaRepo;

    use super::*;
    use animus_core::{content_rating::ContentRating, persona::Persona};
    use uuid::Uuid;

    fn make_persona(name: &str, content_rating: ContentRating) -> Persona {
        Persona {
            id: Uuid::now_v7(),
            name: name.to_owned(),
            description: String::new(),
            personality: String::new(),
            scenario: String::new(),
            first_message: String::new(),
            message_example: String::new(),
            avatar_url: None,
            background_url: None,
            content_rating,
            model: None,
            raw_card: None,
        }
    }

    async fn seed_conversation(pool: &SqlitePool) -> Uuid {
        // 1. Créer d'abord une persona valide
        let persona = make_persona("Toto", ContentRating::Pg);
        let pool_clone = pool.clone();
        let persona_repo = PersonaRepo::new(pool_clone);
        persona_repo.insert(&persona).await.unwrap();

        // 2. Créer la conversation avec un persona_id valide
        let conv_id = Uuid::now_v7();
        let conv_id_str = conv_id.to_string();
        let persona_id_str = persona.id.to_string();
        sqlx::query!(
            r#"
            INSERT INTO conversations (id, persona_id, created_at, updated_at)
            VALUES (?, ?, unixepoch(), unixepoch())
            "#,
            conv_id_str,
            persona_id_str,
        )
        .execute(pool)
        .await
        .unwrap();

        conv_id
    }

    #[sqlx::test(migrator = "crate::summary_repo::MIGRATOR")]
    async fn fetch_latest_summary_returns_none_when_empty(pool: SqlitePool) {
        let repo = SummaryRepo::new(pool);
        let result = repo.find_latest(Uuid::now_v7()).await.unwrap();
        assert!(result.is_none());
    }

    #[sqlx::test(migrator = "crate::summary_repo::MIGRATOR")]
    async fn insert_summary(pool: SqlitePool) {
        let repo = SummaryRepo::new(pool);
        let conversation_id = seed_conversation(&repo.pool).await;
        let summary = Summary {
            id: Uuid::now_v7(),
            conversation_id,
            content: "test".to_string(),
            message_range_start: Uuid::now_v7(),
            message_range_end: Uuid::now_v7(),
        };
        repo.insert(&summary).await.unwrap();
        let result = repo.find_latest(summary.conversation_id).await.unwrap();
        assert!(result.is_some());

        let saved = result.unwrap();
        assert_eq!(saved.conversation_id, conversation_id);
        assert_eq!(saved.content, "test");
    }
}
