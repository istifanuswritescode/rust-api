
use axum::{
    extract::State,
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};

use sqlx::sqlite::SqlitePoolOptions;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

#[derive(Serialize, sqlx::FromRow, Clone)]
struct User {
    id: u32,
    name: String,
}

#[derive(serde::Deserialize)]
struct CreateUser {
    name: String,
}

#[derive(Serialize)]
struct ApiError {
    error: String,
}

#[tokio::main]
async fn main() {

   let pool = SqlitePoolOptions::new()
    .connect("sqlite://users.db")
    .await
    .unwrap();

    sqlx::query(
    r#"
    CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL
    )
    "#,
)
.execute(&pool)
.await
.unwrap();

    let app = Router::new()
    .route("/", get(|| async { "Hello from Rust!" }))
    .route("/users", get(get_users).post(create_user))
    .route("/users/{id}", delete(delete_user).put(update_user))
    .with_state(pool);

    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server running at http://127.0.0.1:3000");

    axum::serve(listener, app)
        .await
        .unwrap();
}

async fn get_users(
    State(pool): State<AppState>,
) -> Result<Json<Vec<User>>, (StatusCode, Json<ApiError>)> {
    let users = sqlx::query_as::<_, User>(
        "SELECT id, name FROM users ORDER BY id"
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: "Failed to fetch users".to_string(),
            }),
        )
    })?;

    Ok(Json(users))
}

async fn create_user(
    State(pool): State<AppState>,
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), (StatusCode, Json<ApiError>)> {
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "Name cannot be empty".to_string(),
            }),
        ));
    }

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (name) VALUES (?) RETURNING id, name"
    )
    .bind(payload.name.trim())
    .fetch_one(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: "Failed to create user".to_string(),
            }),
        )
    })?;

    Ok((StatusCode::CREATED, Json(user)))
}

async fn delete_user(
    State(pool): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<u32>,
) -> (StatusCode, String) {
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await;

    match result {
        Ok(result) if result.rows_affected() > 0 => {
            (
                StatusCode::OK,
                format!("User {} deleted", id),
            )
        }
        Ok(_) => {
            (
                StatusCode::NOT_FOUND,
                format!("User {} not found", id),
            )
        }
        Err(_) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete user".to_string(),
            )
        }
    }
}

async fn update_user(
    State(pool): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<u32>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<User>, (StatusCode, Json<ApiError>)> {
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "Name cannot be empty".to_string(),
            }),
        ));
    }

    let user = sqlx::query_as::<_, User>(
        "UPDATE users SET name = ? WHERE id = ? RETURNING id, name"
    )
    .bind(payload.name.trim())
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: "Failed to update user".to_string(),
            }),
        )
    })?;

    match user {
        Some(user) => Ok(Json(user)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiError {
                error: "User not found".to_string(),
            }),
        )),
    }
}

type AppState = sqlx::SqlitePool;