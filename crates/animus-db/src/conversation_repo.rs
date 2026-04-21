use crate::Conversation;
use uuid::Uuid;
use sqlx::SqlitePool;

pub struct ConversationRepo {
  pool: SqlitePool,
}

impl ConversationRepo {
  pub fn new(pool: SqlitePool) -> Self {
    Self { pool }
  }

  async fn insert(&self, conv: &Conversation) -> Result<(), sqlx::Error> {
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

  async fn find_by_id(&self, id: Uuid) -> Result<Option<Conversation>, sqlx::Error> {
    let id_string = id.to_string();
    let row = sqlx::query!(
      r#"SELECT id, persona_id, created_at FROM conversations WHERE id = ?"#,
      id_string
    )
    .fetch_optional(&self.pool)
    .await?;

    Ok(row.map(|r| Conversation {
      id: Uuid::parse_str(r.id.as_deref().expect("null id in DB")).expect("invalid UUID in DB"),
      persona_id: Uuid::parse_str(&r.persona_id).expect("invalid UUID in DB"),
      created_at: r.created_at,
    }))
  }
}

// Tests
#[cfg(test)]
mod tests {

  use super::*;
  use crate::{persona_repo::PersonaRepo, ContentRating, Persona};
  use sqlx::SqlitePool;
  use uuid::Uuid;

  /// Helper insert_test_persona creates a real persona in the test DB before inserting the
  /// conversation. insert_unknown_persona_id stays untouched — it still tests FK violation
  /// with a random UUID.
  async fn insert_test_persona(pool: &SqlitePool) -> Uuid {
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
    };
    PersonaRepo::new(pool.clone()).insert(&persona).await.unwrap();
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
