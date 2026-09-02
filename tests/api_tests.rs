use serde::Deserialize;
use rust_api::{create_app, hash_password, verify_password};

use tower::ServiceExt;
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};

#[derive(Deserialize)]
struct TestUser {
    id: u32,
    name: String,
}

async fn setup_database() -> sqlx::SqlitePool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .unwrap();

    sqlx::query(
        r#"
   CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    password_hash TEXT NOT NULL DEFAULT 'test-password-hash'
)
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

#[tokio::test]
async fn get_users_returns_success() {
    let pool = setup_database().await;
    
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
       let pool = setup_database().await;
    let app = create_app(pool);

    let request = axum::http::Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
       .body(axum::body::Body::from(
    r#"{"name":"John","password":"secret123"}"#,
))
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
    let pool = setup_database().await;

    let app = create_app(pool);

    let request = axum::http::Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
       .body(axum::body::Body::from(
    r#"{"name":"","password":"secret123"}"#,
))
        .unwrap();

    let response = tower::ServiceExt::oneshot(app, request)
        .await
        .unwrap();

    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn get_user_returns_user() {
    let pool = setup_database().await;

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
    let pool = setup_database().await;

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
    let pool = setup_database().await;

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
    let pool = setup_database().await;


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
    let pool = setup_database().await;

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
    let pool = setup_database().await;

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

#[tokio::test]
async fn update_user_returns_not_found_for_missing_user() {
    let pool = setup_database().await;
    let app = create_app(pool);

    let request = axum::http::Request::builder()
        .method("PUT")
        .uri("/users/999")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"name":"Alice"}"#))
        .unwrap();

    let response = tower::ServiceExt::oneshot(app, request)
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn delete_user_returns_not_found_for_missing_user() {
    let pool = setup_database().await;
    let app = create_app(pool);

    let request = axum::http::Request::builder()
        .method("DELETE")
        .uri("/users/999")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = tower::ServiceExt::oneshot(app, request)
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
}

#[test]
fn password_is_hashed() {
    let password = "secret123";

    let hash1 = hash_password(password).unwrap();
    let hash2 = hash_password(password).unwrap();

    assert_ne!(hash1, password);
    assert_ne!(hash2, password);
    assert_ne!(hash1, hash2);
}

#[test]
fn password_verification_works() {
    let password = "secret123";

    let password_hash = hash_password(password).unwrap();

    assert!(verify_password(password, &password_hash).unwrap());
    assert!(!verify_password("wrong-password", &password_hash).unwrap());
}

#[tokio::test]
async fn login_returns_token_for_correct_password() {
    let pool = setup_database().await;

    unsafe {
    std::env::set_var("JWT_SECRET", "test-secret");
}

    let password_hash = hash_password("secret123").unwrap();

    sqlx::query(
        "INSERT INTO users (name, password_hash) VALUES (?, ?)"
    )
    .bind("Alice")
    .bind(&password_hash)
    .execute(&pool)
    .await
    .unwrap();

    let app = create_app(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name":"Alice","password":"secret123"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let body = String::from_utf8(body.to_vec()).unwrap();

  assert!(body.contains("\"token\""));
}

#[tokio::test]
async fn login_rejects_unknown_user() {
    let pool = setup_database().await;

    let app = create_app(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name":"Unknown","password":"secret123"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_rejects_wrong_password() {
    let pool = setup_database().await;

    let password_hash = hash_password("secret123").unwrap();

    sqlx::query(
        "INSERT INTO users (name, password_hash)
         VALUES (?, ?)"
    )
    .bind("Alice")
        .bind(&password_hash)
        .execute(&pool)
        .await
        .unwrap();
}

#[test]
fn create_token_returns_token() {
    let token = rust_api::create_token(1, "test-secret")
        .expect("token should be created");

    assert!(!token.is_empty());
}

#[test]
fn verify_token_returns_claims() {
    let token = rust_api::create_token(42, "test-secret")
        .expect("token should be created");

    let claims = rust_api::verify_token(&token, "test-secret")
        .expect("token should be valid");

    assert_eq!(claims.sub, 42);
}