use serde::Serialize;

use axum::{
     extract::State,
    http::StatusCode,
    routing::get,
    Json, Router,
};

#[derive(Serialize, sqlx::FromRow, Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
}

#[derive(Serialize)]
pub struct ApiError {
    pub error: String,
}

#[derive(serde::Deserialize)]
pub struct CreateUser {
    pub name: String,
}

#[derive(Serialize)]
pub struct ApiMessage {
    pub message: String,
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

pub fn create_app(pool: AppState) -> Router {
    Router::new()
        .route("/", get(|| async { "Hello from Rust!" }))
        .route("/users", get(get_users).post(create_user))
        .route("/users/{id}", get(get_user).delete(delete_user).put(update_user))
        .with_state(pool)
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

async fn get_user(
    State(pool): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<u32>,
) -> Result<Json<User>, (StatusCode, Json<ApiError>)> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, name FROM users WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: "Failed to fetch user".to_string(),
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

async fn delete_user(
    State(pool): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<u32>,
) -> Result<(StatusCode, Json<ApiMessage>), (StatusCode, Json<ApiError>)> {
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await;

    match result {
      Ok(result) if result.rows_affected() > 0 => {
    Ok((
        StatusCode::OK,
        Json(ApiMessage {
            message: format!("User {} deleted", id),
        }),
    ))
}
       Ok(_) => {
    Err((
        StatusCode::NOT_FOUND,
        Json(ApiError {
            error: "User not found".to_string(),
        }),
    ))
}
      Err(_) => {
    Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError {
            error: "Failed to delete user".to_string(),
        }),
    ))
}
    }
}


pub type AppState = sqlx::SqlitePool;