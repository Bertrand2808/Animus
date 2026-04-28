mod error;
mod routes;
mod state;
mod summary_trigger;

use animus_db::{
    persona_repo::PersonaRepo, summary_repo::SummaryRepo, ConversationRepo, MessageRepo,
};
use animus_llm::ollama::OllamaClient;
use anyhow::Context;
use dirs::home_dir;
use sqlx::sqlite::SqliteConnectOptions;
use std::path::PathBuf;
use std::str::FromStr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // TODO: handle this warning
    let home = home_dir().context("failed to get home directory")?;
    let log_dir = PathBuf::from("~/.animus/logs");
    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir).context("failed to create log directory")?;
    }

    dotenvy::dotenv().ok();

    let file_appender = tracing_appender::rolling::daily(&log_dir, "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "animus_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .init();

    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let opts = SqliteConnectOptions::from_str(&database_url)?.create_if_missing(true);
    let pool = sqlx::SqlitePool::connect_with(opts).await?;
    sqlx::migrate!("../../crates/animus-db/migrations")
        .run(&pool)
        .await?;

    tracing::info!("database ready");

    let ollama_url =
        std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_owned());
    let model_name = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "gemma4".to_owned());
    let app_state = state::AppState {
        personas: PersonaRepo::new(pool.clone()),
        conversations: ConversationRepo::new(pool.clone()),
        messages: MessageRepo::new(pool.clone()),
        summaries: SummaryRepo::new(pool),
        ollama: OllamaClient::new(ollama_url),
        model_name,
    };

    let app = routes::personas::router()
        .merge(routes::conversations::router())
        .merge(routes::summary::router())
        .merge(routes::health::router())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8082").await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
