use rust_api::create_app;
use sqlx::sqlite::SqlitePoolOptions;

#[tokio::test]
async fn get_users_returns_success() {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();
    
    sqlx::query("INSERT INTO users (name) VALUES (?)")
    .bind("Daniel")
    .execute(&pool)
    .await
    .unwrap();

    let app = create_app(pool);

    let response = axum::http::Request::builder()
        .uri("/users")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = tower::ServiceExt::oneshot(app, response)
    .await
    .unwrap();

assert_eq!(response.status(), 200);

let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();

let body = String::from_utf8(body.to_vec()).unwrap();

assert!(body.contains("Daniel"));
}