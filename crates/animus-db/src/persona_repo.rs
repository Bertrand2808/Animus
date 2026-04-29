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
        tracing::debug!(
            target: "personas",
            persona_id = %persona.id,
            persona_name = %persona.name,
            has_model_instructions = !persona.model_instructions.trim().is_empty(),
            has_appearance = !persona.appearance.trim().is_empty(),
            has_speech_style = !persona.speech_style.trim().is_empty(),
            has_character_goals = !persona.character_goals.trim().is_empty(),
            has_post_history_instructions = !persona.post_history_instructions.trim().is_empty(),
            response_length_limit = persona.response_length_limit,
            temperature = persona.temperature,
            repeat_penalty = persona.repeat_penalty,
            instruction_template = %persona.instruction_template,
            "inserting persona in db"
        );
        sqlx::query!(
            r#"
      INSERT INTO personas (
        id, name, description, personality, scenario, first_message,
        message_example, avatar_url, background_url, content_rating, model,
        raw_card, model_instructions, appearance, speech_style, character_goals,
        post_history_instructions, response_length_limit, temperature, repeat_penalty,
        instruction_template, created_at
      )
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, unixepoch())
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
            persona.model_instructions,
            persona.appearance,
            persona.speech_style,
            persona.character_goals,
            persona.post_history_instructions,
            persona.response_length_limit,
            persona.temperature,
            persona.repeat_penalty,
            persona.instruction_template,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db) if db.is_unique_violation() => RepoError::Duplicate,
            other => other.into(),
        })?;
        tracing::debug!(target: "personas", persona_id = %persona.id, "insert complete");
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
        raw_card,
        model_instructions AS "model_instructions!",
        appearance AS "appearance!",
        speech_style AS "speech_style!",
        character_goals AS "character_goals!",
        post_history_instructions AS "post_history_instructions!",
        response_length_limit AS "response_length_limit!",
        temperature AS "temperature!",
        repeat_penalty AS "repeat_penalty!",
        instruction_template AS "instruction_template!"
      FROM personas WHERE id = ?
      "#,
            id_str
        )
        .fetch_optional(&self.pool)
        .await?;

        tracing::debug!(
            target: "personas",
            persona_id = %id,
            found = row.is_some(),
            "find persona by id complete"
        );

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
                model_instructions: r.model_instructions,
                appearance: r.appearance,
                speech_style: r.speech_style,
                character_goals: r.character_goals,
                post_history_instructions: r.post_history_instructions,
                response_length_limit: r.response_length_limit,
                temperature: r.temperature,
                repeat_penalty: r.repeat_penalty,
                instruction_template: r.instruction_template,
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
            raw_card,
            model_instructions AS "model_instructions!",
            appearance AS "appearance!",
            speech_style AS "speech_style!",
            character_goals AS "character_goals!",
            post_history_instructions AS "post_history_instructions!",
            response_length_limit AS "response_length_limit!",
            temperature AS "temperature!",
            repeat_penalty AS "repeat_penalty!",
            instruction_template AS "instruction_template!"
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
                        model_instructions: r.model_instructions,
                        appearance: r.appearance,
                        speech_style: r.speech_style,
                        character_goals: r.character_goals,
                        post_history_instructions: r.post_history_instructions,
                        response_length_limit: r.response_length_limit,
                        temperature: r.temperature,
                        repeat_penalty: r.repeat_penalty,
                        instruction_template: r.instruction_template,
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
          raw_card,
          model_instructions AS "model_instructions!",
          appearance AS "appearance!",
          speech_style AS "speech_style!",
          character_goals AS "character_goals!",
          post_history_instructions AS "post_history_instructions!",
          response_length_limit AS "response_length_limit!",
          temperature AS "temperature!",
          repeat_penalty AS "repeat_penalty!",
          instruction_template AS "instruction_template!"
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
                    model_instructions: r.model_instructions,
                    appearance: r.appearance,
                    speech_style: r.speech_style,
                    character_goals: r.character_goals,
                    post_history_instructions: r.post_history_instructions,
                    response_length_limit: r.response_length_limit,
                    temperature: r.temperature,
                    repeat_penalty: r.repeat_penalty,
                    instruction_template: r.instruction_template,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?,
        };
        tracing::debug!(
            target: "personas",
            content_rating = content_rating.map(|cr| cr.to_string()).as_deref(),
            count = rows.len(),
            "list personas complete"
        );
        Ok(rows)
    }

    pub async fn update(&self, persona: &Persona) -> Result<bool, RepoError> {
        tracing::debug!(
            target: "personas",
            persona_id = %persona.id,
            persona_name = %persona.name,
            has_model_instructions = !persona.model_instructions.trim().is_empty(),
            has_appearance = !persona.appearance.trim().is_empty(),
            has_speech_style = !persona.speech_style.trim().is_empty(),
            has_character_goals = !persona.character_goals.trim().is_empty(),
            has_post_history_instructions = !persona.post_history_instructions.trim().is_empty(),
            response_length_limit = persona.response_length_limit,
            temperature = persona.temperature,
            repeat_penalty = persona.repeat_penalty,
            instruction_template = %persona.instruction_template,
            "updating persona in db"
        );
        let id = persona.id.to_string();
        let content_rating = persona.content_rating.to_string();
        let result = sqlx::query!(
            r#"
            UPDATE personas SET
              name = ?, description = ?, personality = ?, scenario = ?,
              first_message = ?, message_example = ?, avatar_url = ?,
              background_url = ?, content_rating = ?, model = ?,
              model_instructions = ?, appearance = ?, speech_style = ?,
              character_goals = ?, post_history_instructions = ?,
              response_length_limit = ?, temperature = ?, repeat_penalty = ?,
              instruction_template = ?
            WHERE id = ?
            "#,
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
            persona.model_instructions,
            persona.appearance,
            persona.speech_style,
            persona.character_goals,
            persona.post_history_instructions,
            persona.response_length_limit,
            persona.temperature,
            persona.repeat_penalty,
            persona.instruction_template,
            id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db) if db.is_unique_violation() => RepoError::Duplicate,
            other => other.into(),
        })?;
        let found = result.rows_affected() > 0;
        tracing::debug!(target: "personas", persona_id = %persona.id, found = found, "update complete");
        Ok(found)
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
    use animus_core::{
        content_rating::ContentRating,
        persona::{
            DEFAULT_INSTRUCTION_TEMPLATE, DEFAULT_REPEAT_PENALTY, DEFAULT_RESPONSE_LENGTH_LIMIT,
            DEFAULT_TEMPERATURE, Persona,
        },
    };
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
            model_instructions: String::new(),
            appearance: String::new(),
            speech_style: String::new(),
            character_goals: String::new(),
            post_history_instructions: String::new(),
            response_length_limit: DEFAULT_RESPONSE_LENGTH_LIMIT,
            temperature: DEFAULT_TEMPERATURE,
            repeat_penalty: DEFAULT_REPEAT_PENALTY,
            instruction_template: DEFAULT_INSTRUCTION_TEMPLATE.to_owned(),
        }
    }

    #[sqlx::test(migrator = "crate::persona_repo::MIGRATOR")]
    async fn insert_and_fetch_persona(pool: SqlitePool) {
        let repo = PersonaRepo::new(pool);
        let mut persona = make_persona("Aria", ContentRating::Pg);
        persona.model_instructions = "Stay in character".to_owned();
        persona.appearance = "Silver hair".to_owned();
        persona.speech_style = "Concise".to_owned();
        persona.character_goals = "Help the user".to_owned();
        persona.post_history_instructions = "Use recent context".to_owned();
        persona.response_length_limit = 900;
        persona.temperature = 0.8;
        persona.repeat_penalty = 1.2;
        persona.instruction_template = "cinematic".to_owned();
        repo.insert(&persona).await.unwrap();
        let fetched = repo.find_by_id(persona.id).await.unwrap().unwrap();
        assert_eq!(fetched.name, "Aria");
        assert_eq!(fetched.content_rating, ContentRating::Pg);
        assert_eq!(fetched.model_instructions, "Stay in character");
        assert_eq!(fetched.appearance, "Silver hair");
        assert_eq!(fetched.speech_style, "Concise");
        assert_eq!(fetched.character_goals, "Help the user");
        assert_eq!(fetched.post_history_instructions, "Use recent context");
        assert_eq!(fetched.response_length_limit, 900);
        assert_eq!(fetched.temperature, 0.8);
        assert_eq!(fetched.repeat_penalty, 1.2);
        assert_eq!(fetched.instruction_template, "cinematic");
    }

    #[sqlx::test(migrator = "crate::persona_repo::MIGRATOR")]
    async fn inserted_old_shape_persona_fetches_structured_defaults(pool: SqlitePool) {
        let id = Uuid::now_v7();
        let id_str = id.to_string();
        sqlx::query!(
            r#"
            INSERT INTO personas (
                id, name, description, personality, scenario, first_message,
                message_example, avatar_url, background_url, content_rating, model,
                raw_card, created_at
            )
            VALUES (?, 'Old', '', '', '', '', '', NULL, NULL, 'pg', NULL, NULL, unixepoch())
            "#,
            id_str
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = PersonaRepo::new(pool);
        let fetched = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(fetched.model_instructions, "");
        assert_eq!(fetched.appearance, "");
        assert_eq!(fetched.speech_style, "");
        assert_eq!(fetched.character_goals, "");
        assert_eq!(fetched.post_history_instructions, "");
        assert_eq!(fetched.response_length_limit, DEFAULT_RESPONSE_LENGTH_LIMIT);
        assert_eq!(fetched.temperature, DEFAULT_TEMPERATURE);
        assert_eq!(fetched.repeat_penalty, DEFAULT_REPEAT_PENALTY);
        assert_eq!(fetched.instruction_template, DEFAULT_INSTRUCTION_TEMPLATE);
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

    #[sqlx::test(migrator = "crate::persona_repo::MIGRATOR")]
    async fn update_existing_returns_true(pool: SqlitePool) {
        let repo = PersonaRepo::new(pool);
        let mut p = make_persona("Aria", ContentRating::Pg);
        repo.insert(&p).await.unwrap();
        p.name = "Aria Renamed".to_owned();
        p.model_instructions = "Updated instructions".to_owned();
        p.response_length_limit = 300;
        p.temperature = 0.4;
        p.repeat_penalty = 1.3;
        p.instruction_template = "updated".to_owned();
        assert!(repo.update(&p).await.unwrap());
        let fetched = repo.find_by_id(p.id).await.unwrap().unwrap();
        assert_eq!(fetched.name, "Aria Renamed");
        assert_eq!(fetched.model_instructions, "Updated instructions");
        assert_eq!(fetched.response_length_limit, 300);
        assert_eq!(fetched.temperature, 0.4);
        assert_eq!(fetched.repeat_penalty, 1.3);
        assert_eq!(fetched.instruction_template, "updated");
    }

    #[sqlx::test(migrator = "crate::persona_repo::MIGRATOR")]
    async fn update_not_found_returns_false(pool: SqlitePool) {
        let repo = PersonaRepo::new(pool);
        let p = make_persona("Ghost", ContentRating::Pg);
        assert!(!repo.update(&p).await.unwrap());
    }

    #[sqlx::test(migrator = "crate::persona_repo::MIGRATOR")]
    async fn update_duplicate_name_returns_error(pool: SqlitePool) {
        let repo = PersonaRepo::new(pool);
        let a = make_persona("Aria", ContentRating::Pg);
        let mut b = make_persona("Bob", ContentRating::Pg);
        repo.insert(&a).await.unwrap();
        repo.insert(&b).await.unwrap();
        b.name = "Aria".to_owned();
        let err = repo.update(&b).await.unwrap_err();
        assert!(matches!(err, RepoError::Duplicate));
    }
}
