use animus_core::{content_rating::ContentRating, persona::Persona};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PersonaRepo {
  pool: SqlitePool,
}

impl PersonaRepo {
  pub fn new(pool: SqlitePool) -> Self {
    Self { pool }
  }

  pub async fn insert(&self, persona: &Persona) -> Result<(), sqlx::Error> {
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
    .await?;
    Ok(())
  }

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
        raw_card    AS "raw_card!"
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
}

#[cfg(test)]
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

#[cfg(test)]
mod tests {
  use super::*;
  use animus_core::{content_rating::ContentRating, persona::Persona};
  use uuid::Uuid;

  #[sqlx::test(migrator = "crate::persona_repo::MIGRATOR")]
  async fn insert_and_fetch_persona(pool: SqlitePool) {
    let repo = PersonaRepo::new(pool);

    let persona = Persona {
      id: Uuid::now_v7(),
      name: "Aria".to_owned(),
      description: "A test persona".to_owned(),
      personality: "Calm".to_owned(),
      scenario: "A library".to_owned(),
      first_message: "Hello !".to_owned(),
      message_example: "".to_owned(),
      avatar_url: None,
      background_url: None,
      content_rating: ContentRating::Pg,
      model: None,
      raw_card: "{}".to_owned(),
    };

    repo.insert(&persona).await.unwrap();

    let fetched = repo.find_by_id(persona.id).await.unwrap().unwrap();
    assert_eq!(fetched.name, "Aria");
    assert_eq!(fetched.content_rating, ContentRating::Pg);
  }
}
