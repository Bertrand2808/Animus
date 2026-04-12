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

    pub async fn find_latest(
        &self,
        conversation_id: Uuid,
    ) -> Result<Option<Summary>, sqlx::Error> {
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
}

#[cfg(test)]
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[sqlx::test(migrator = "crate::summary_repo::MIGRATOR")]
    async fn fetch_latest_summary_returns_none_when_empty(pool: SqlitePool) {
        let repo = SummaryRepo::new(pool);
        let result = repo.find_latest(Uuid::now_v7()).await.unwrap();
        assert!(result.is_none());
    }
}
