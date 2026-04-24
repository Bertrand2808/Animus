use animus_core::{content_rating::ContentRating, persona::Persona};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("a persona with this name already exists")]
    Duplicate,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Clone)]
pub struct PersonaRepo {
    pool: SqlitePool,
}

impl PersonaRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, persona: &Persona) -> Result<(), RepoError> {
        let id = persona.id.to_string();
        let content_rating = persona.content_rating.to_string();
        sqlx::query!(
            r#"
      INSERT INTO personas (
        id, name, description, personality, scenario, first_message,
        message_example, avatar_url, background_url, content_rating, model,
        raw_card, created_at
      )
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, unixepoch())
      "#,
            id,
            persona.name,
            persona.description,
            persona.personality,
            persona.scenario,
            persona.first_message,
            persona.message_example,
            persona.avatar_url,
            persona.background_url,
            content_rating,
            persona.model,
            persona.raw_card,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db) if db.is_unique_violation() => RepoError::Duplicate,
            other => other.into(),
        })?;
        Ok(())
    }

    /// Find a persona by its ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Persona>, sqlx::Error> {
        let id_str = id.to_string();
        let row = sqlx::query!(
            r#"
      SELECT
        id          AS "id!",
        name        AS "name!",
        description AS "description!",
        personality AS "personality!",
        scenario    AS "scenario!",
        first_message  AS "first_message!",
        message_example AS "message_example!",
        avatar_url,
        background_url,
        content_rating AS "content_rating!",
        model,
        raw_card
      FROM personas WHERE id = ?
      "#,
            id_str
        )
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| {
            Ok(Persona {
                id: r.id.parse().map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                name: r.name,
                description: r.description,
                personality: r.personality,
                scenario: r.scenario,
                first_message: r.first_message,
                message_example: r.message_example,
                avatar_url: r.avatar_url,
                background_url: r.background_url,
                content_rating: r
                    .content_rating
                    .parse::<ContentRating>()
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                model: r.model,
                raw_card: r.raw_card,
            })
        })
        .transpose()
    }

    pub async fn find_all(
        &self,
        content_rating: Option<ContentRating>,
    ) -> Result<Vec<Persona>, sqlx::Error> {
        let rows = match content_rating {
            Some(cr) => {
                let cr_str = cr.to_string();
                sqlx::query!(
                    r#"
          SELECT
            id          AS "id!",
            name        AS "name!",
            description AS "description!",
            personality AS "personality!",
            scenario    AS "scenario!",
            first_message  AS "first_message!",
            message_example AS "message_example!",
            avatar_url,
            background_url,
            content_rating AS "content_rating!",
            model,
            raw_card
          FROM personas WHERE content_rating = ? ORDER BY name
          "#,
                    cr_str
                )
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|r| {
                    Ok(Persona {
                        id: r.id.parse().map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                        name: r.name,
                        description: r.description,
                        personality: r.personality,
                        scenario: r.scenario,
                        first_message: r.first_message,
                        message_example: r.message_example,
                        avatar_url: r.avatar_url,
                        background_url: r.background_url,
                        content_rating: r
                            .content_rating
                            .parse::<ContentRating>()
                            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                        model: r.model,
                        raw_card: r.raw_card,
                    })
                })
                .collect::<Result<Vec<_>, sqlx::Error>>()?
            }
            None => sqlx::query!(
                r#"
        SELECT
          id          AS "id!",
          name        AS "name!",
          description AS "description!",
          personality AS "personality!",
          scenario    AS "scenario!",
          first_message  AS "first_message!",
          message_example AS "message_example!",
          avatar_url,
          background_url,
          content_rating AS "content_rating!",
          model,
          raw_card
        FROM personas ORDER BY name
        "#
            )
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r| {
                Ok(Persona {
                    id: r.id.parse().map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                    name: r.name,
                    description: r.description,
                    personality: r.personality,
                    scenario: r.scenario,
                    first_message: r.first_message,
                    message_example: r.message_example,
                    avatar_url: r.avatar_url,
                    background_url: r.background_url,
                    content_rating: r
                        .content_rating
                        .parse::<ContentRating>()
                        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                    model: r.model,
                    raw_card: r.raw_card,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?,
        };
        Ok(rows)
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let id_str = id.to_string();
        let result = sqlx::query!("DELETE FROM personas WHERE id = ?", id_str)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

#[cfg(test)]
mod tests {
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

    #[sqlx::test(migrator = "crate::persona_repo::MIGRATOR")]
    async fn insert_and_fetch_persona(pool: SqlitePool) {
        let repo = PersonaRepo::new(pool);
        let persona = make_persona("Aria", ContentRating::Pg);
        repo.insert(&persona).await.unwrap();
        let fetched = repo.find_by_id(persona.id).await.unwrap().unwrap();
        assert_eq!(fetched.name, "Aria");
        assert_eq!(fetched.content_rating, ContentRating::Pg);
    }

    #[sqlx::test(migrator = "crate::persona_repo::MIGRATOR")]
    async fn insert_duplicate_name_returns_duplicate_error(pool: SqlitePool) {
        let repo = PersonaRepo::new(pool);
        let p1 = make_persona("Aria", ContentRating::Pg);
        let mut p2 = make_persona("Aria", ContentRating::Mature);
        p2.id = Uuid::now_v7();
        repo.insert(&p1).await.unwrap();
        let err = repo.insert(&p2).await.unwrap_err();
        assert!(matches!(err, RepoError::Duplicate));
    }

    #[sqlx::test(migrator = "crate::persona_repo::MIGRATOR")]
    async fn find_by_id_not_found(pool: SqlitePool) {
        let repo = PersonaRepo::new(pool);
        let result = repo.find_by_id(Uuid::now_v7()).await.unwrap();
        assert!(result.is_none());
    }

    #[sqlx::test(migrator = "crate::persona_repo::MIGRATOR")]
    async fn find_all_returns_all(pool: SqlitePool) {
        let repo = PersonaRepo::new(pool);
        repo.insert(&make_persona("A", ContentRating::Pg))
            .await
            .unwrap();
        repo.insert(&make_persona("B", ContentRating::Nsfw))
            .await
            .unwrap();
        let all = repo.find_all(None).await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[sqlx::test(migrator = "crate::persona_repo::MIGRATOR")]
    async fn find_all_filtered_by_content_rating(pool: SqlitePool) {
        let repo = PersonaRepo::new(pool);
        repo.insert(&make_persona("A", ContentRating::Pg))
            .await
            .unwrap();
        repo.insert(&make_persona("B", ContentRating::Nsfw))
            .await
            .unwrap();
        let nsfw = repo.find_all(Some(ContentRating::Nsfw)).await.unwrap();
        assert_eq!(nsfw.len(), 1);
        assert_eq!(nsfw[0].name, "B");
    }

    #[sqlx::test(migrator = "crate::persona_repo::MIGRATOR")]
    async fn find_all_empty(pool: SqlitePool) {
        let repo = PersonaRepo::new(pool);
        let all = repo.find_all(None).await.unwrap();
        assert!(all.is_empty());
    }

    #[sqlx::test(migrator = "crate::persona_repo::MIGRATOR")]
    async fn find_all_filtered_no_match(pool: SqlitePool) {
        let repo = PersonaRepo::new(pool);
        repo.insert(&make_persona("A", ContentRating::Pg))
            .await
            .unwrap();
        let nsfw = repo.find_all(Some(ContentRating::Nsfw)).await.unwrap();
        assert!(nsfw.is_empty());
    }

    #[sqlx::test(migrator = "crate::persona_repo::MIGRATOR")]
    async fn delete_existing_returns_true(pool: SqlitePool) {
        let repo = PersonaRepo::new(pool);
        let p = make_persona("ToDelete", ContentRating::Pg);
        repo.insert(&p).await.unwrap();
        assert!(repo.delete(p.id).await.unwrap());
        assert!(repo.find_by_id(p.id).await.unwrap().is_none());
    }

    #[sqlx::test(migrator = "crate::persona_repo::MIGRATOR")]
    async fn delete_not_found_returns_false(pool: SqlitePool) {
        let repo = PersonaRepo::new(pool);
        assert!(!repo.delete(Uuid::now_v7()).await.unwrap());
    }
}
