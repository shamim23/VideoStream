use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
};
use sqlx::sqlite::SqlitePoolOptions;

mod api;
mod domain;
mod storage;

use api::{health_check, upload_handler, stream_handler, AppState};
use storage::LocalStorage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Get absolute path for storage
    let storage_path = std::env::current_dir()?.join("storage");
    let db_path = storage_path.join("database.db");
    
    // Ensure storage directory exists (using std::fs before async runtime)
    std::fs::create_dir_all(&storage_path)?;
    
    println!("Storage path: {:?}", storage_path);
    println!("Database path: {:?}", db_path);

    // Initialize database using connect options with create_if_missing
    let connect_options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true);
    
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await?;

    // Run migrations
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS videos (
            id TEXT PRIMARY KEY,
            filename TEXT NOT NULL,
            content_type TEXT NOT NULL,
            size_bytes INTEGER NOT NULL,
            storage_path TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(&db)
    .await?;

    println!("Database initialized");

    // Initialize storage
    let storage: Arc<dyn storage::Storage> = Arc::new(LocalStorage::new(&storage_path));

    let state = Arc::new(AppState { storage, db });

    let cors = CorsLayer::new()
        .allow_origin(["http://localhost:5173".parse().unwrap()])
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(health_check))
        .route("/api/upload", post(upload_handler))
        .route("/api/watch/:id", get(stream_handler))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(1024 * 1024 * 1024)) // 1GB limit
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
