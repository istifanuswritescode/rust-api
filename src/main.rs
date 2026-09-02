use sqlx::sqlite::SqlitePoolOptions;
use tokio::net::TcpListener;
use rust_api::create_app;

#[tokio::main]
async fn main() {

    dotenvy::dotenv().ok();
    let pool = SqlitePoolOptions::new()
        .connect(
    &std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env"),
)
        .await
        .unwrap();

    sqlx::query(
        r#"
       CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    password_hash TEXT NOT NULL
)
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    let app = create_app(pool);

    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server running at http://127.0.0.1:3000");

    axum::serve(listener, app)
        .await
        .unwrap();
}