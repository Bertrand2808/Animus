use animus_core::AppSettings;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct SettingsRepo {
    pool: SqlitePool,
}

impl SettingsRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Insert the default row only when none exists yet.
    /// Called once at startup to seed `default_model` from the env.
    pub async fn init_defaults(&self, default_model: &str) -> Result<(), sqlx::Error> {
        let rows = sqlx::query!(
            "INSERT OR IGNORE INTO app_settings (id, user_name, default_model) VALUES (1, 'User', ?)",
            default_model
        )
        .execute(&self.pool)
        .await?;

        if rows.rows_affected() > 0 {
            tracing::info!(
                target: "settings",
                default_model = %default_model,
                "settings initialized with defaults"
            );
        }
        Ok(())
    }

    /// Fetch the singleton settings row, returning built-in defaults when not yet initialised.
    pub async fn get(&self) -> Result<AppSettings, sqlx::Error> {
        let opt = sqlx::query!(
            r#"SELECT user_name AS "user_name!", default_model AS "default_model!" FROM app_settings WHERE id = 1"#
        )
        .fetch_optional(&self.pool)
        .await?;

        let settings = match opt {
            Some(row) => {
                tracing::debug!(target: "settings", user_name = %row.user_name, "settings fetched");
                AppSettings {
                    user_name: row.user_name,
                    default_model: row.default_model,
                }
            }
            None => {
                // init_defaults has not been called yet; callers that depend on default_model
                // will receive an empty string until the row is seeded.
                tracing::warn!(target: "settings", "settings row absent — using built-in defaults; call init_defaults at startup");
                AppSettings {
                    user_name: "User".to_owned(),
                    default_model: String::new(),
                }
            }
        };

        Ok(settings)
    }

    /// Overwrite the singleton settings row entirely.
    pub async fn upsert(&self, settings: &AppSettings) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT OR REPLACE INTO app_settings (id, user_name, default_model) VALUES (1, ?, ?)",
            settings.user_name,
            settings.default_model,
        )
        .execute(&self.pool)
        .await?;

        tracing::debug!(
            target: "settings",
            user_name = %settings.user_name,
            default_model = %settings.default_model,
            "settings updated"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    pub static MIGRATOR: sqlx::migrate::Migrator =
        sqlx::migrate!("../../crates/animus-db/migrations");

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn init_defaults_creates_row(pool: SqlitePool) {
        let repo = SettingsRepo::new(pool);
        repo.init_defaults("llama3").await.unwrap();
        let s = repo.get().await.unwrap();
        assert_eq!(s.user_name, "User");
        assert_eq!(s.default_model, "llama3");
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn init_defaults_is_idempotent(pool: SqlitePool) {
        let repo = SettingsRepo::new(pool);
        repo.init_defaults("llama3").await.unwrap();
        repo.init_defaults("other-model").await.unwrap();
        let s = repo.get().await.unwrap();
        assert_eq!(s.default_model, "llama3", "second call must not overwrite");
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn upsert_updates_settings(pool: SqlitePool) {
        let repo = SettingsRepo::new(pool);
        repo.init_defaults("gemma4").await.unwrap();

        repo.upsert(&AppSettings {
            user_name: "Bertrand".to_owned(),
            default_model: "mistral".to_owned(),
        })
        .await
        .unwrap();

        let s = repo.get().await.unwrap();
        assert_eq!(s.user_name, "Bertrand");
        assert_eq!(s.default_model, "mistral");
    }
}
