use serde::Deserialize;
use rust_api::create_app;
use sqlx::sqlite::SqlitePoolOptions;

#[derive(Deserialize)]
struct TestUser {
    id: u32,
    name: String,
}

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

let users: Vec<TestUser> =
    serde_json::from_slice(&body).unwrap();

assert_eq!(users.len(), 1);
assert_eq!(users[0].name, "Daniel");
assert_eq!(users[0].id, 1);
}

#[tokio::test]
async fn create_user_returns_created_user() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
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

    let app = create_app(pool);

    let request = axum::http::Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"name":"John"}"#))
        .unwrap();

    let response = tower::ServiceExt::oneshot(app, request)
        .await
        .unwrap();

    assert_eq!(response.status(), 201);

let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();

let user: TestUser =
    serde_json::from_slice(&body).unwrap();

assert_eq!(user.id, 1);
assert_eq!(user.name, "John");
}

#[tokio::test]
async fn create_user_rejects_empty_name() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
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

    let app = create_app(pool);

    let request = axum::http::Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"name":""}"#))
        .unwrap();

    let response = tower::ServiceExt::oneshot(app, request)
        .await
        .unwrap();

    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn get_user_returns_user() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
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
        .bind("Alice")
        .execute(&pool)
        .await
        .unwrap();

    let app = create_app(pool);

    let request = axum::http::Request::builder()
        .method("GET")
        .uri("/users/1")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = tower::ServiceExt::oneshot(app, request)
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let user: TestUser =
        serde_json::from_slice(&body).unwrap();

    assert_eq!(user.id, 1);
    assert_eq!(user.name, "Alice");
}

#[tokio::test]
async fn get_user_returns_not_found_for_missing_user() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
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

    let app = create_app(pool);

    let request = axum::http::Request::builder()
        .method("GET")
        .uri("/users/999")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = tower::ServiceExt::oneshot(app, request)
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn update_user_returns_updated_user() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
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
        .bind("Alice")
        .execute(&pool)
        .await
        .unwrap();

    let app = create_app(pool);

    let request = axum::http::Request::builder()
        .method("PUT")
        .uri("/users/1")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"name":"Alice Updated"}"#))
        .unwrap();

    let response = tower::ServiceExt::oneshot(app, request)
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let user: TestUser =
        serde_json::from_slice(&body).unwrap();

    assert_eq!(user.id, 1);
    assert_eq!(user.name, "Alice Updated");
}

#[tokio::test]
async fn delete_user_returns_success() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
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
        .bind("Alice")
        .execute(&pool)
        .await
        .unwrap();

    let app = create_app(pool);

    let request = axum::http::Request::builder()
        .method("DELETE")
        .uri("/users/1")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = tower::ServiceExt::oneshot(app, request)
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn delete_user_removes_user() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
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
        .bind("Alice")
        .execute(&pool)
        .await
        .unwrap();

    let app = create_app(pool);

    let request = axum::http::Request::builder()
        .method("DELETE")
        .uri("/users/1")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = tower::ServiceExt::oneshot(app.clone(), request)
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let request = axum::http::Request::builder()
        .method("GET")
        .uri("/users/1")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = tower::ServiceExt::oneshot(app, request)
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn update_user_rejects_empty_name() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
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
        .bind("Alice")
        .execute(&pool)
        .await
        .unwrap();

    let app = create_app(pool);

    let request = axum::http::Request::builder()
        .method("PUT")
        .uri("/users/1")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"name":""}"#))
        .unwrap();

    let response = tower::ServiceExt::oneshot(app, request)
        .await
        .unwrap();

    assert_eq!(response.status(), 400);
}