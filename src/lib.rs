use serde::Serialize;

use axum::http::HeaderMap;

use argon2::{
    password_hash::{
        PasswordHasher,
        PasswordVerifier,
        SaltString,
    },
    Argon2,
};

use argon2::password_hash::rand_core::OsRng;

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

#[derive(sqlx::FromRow)]
struct UserRecord {
    pub id: u32,
    pub password_hash: String,
}

use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Deserialize;


#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: u32,
    pub exp: usize,
}

pub fn create_token(user_id: u32, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims {
        sub: user_id,
        exp: 2_000_000_000,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn verify_token(
    token: &str,
    secret: &str,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = jsonwebtoken::Validation::default();

    let token_data = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;

    Ok(token_data.claims)
}

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

pub fn verify_password(
    password: &str,
    password_hash: &str,
) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash =
        argon2::password_hash::PasswordHash::new(password_hash)?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[derive(Serialize)]
pub struct ApiError {
    pub error: String,
}

#[derive(serde::Deserialize)]
pub struct CreateUser {
    pub name: String,
    pub password: String,
}

#[derive(serde::Deserialize)]
pub struct UpdateUser {
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

    let password_hash = hash_password(&payload.password).map_err(|_| {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError {
            error: "Failed to hash password".to_string(),
        }),
    )
})?;

    let user = sqlx::query_as::<_, User>(
       "INSERT INTO users (name, password_hash) VALUES (?, ?) RETURNING id, name"
    )
   .bind(payload.name.trim())
.bind(&password_hash)
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

#[derive(serde::Deserialize)]
pub struct LoginUser {
    pub name: String,
    pub password: String,
}

#[derive(serde::Serialize)]
pub struct LoginResponse {
    pub token: String,
}

async fn login_user(
    State(pool): State<AppState>,
    Json(payload): Json<LoginUser>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ApiError>)> {
    let user = sqlx::query_as::<_, UserRecord>(
        "SELECT id, password_hash FROM users WHERE name = ?"
    )
    .bind(payload.name.trim())
    .fetch_optional(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: "Failed to find user".to_string(),
            }),
        )
    })?;

    let user = match user {
        Some(user) => user,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ApiError {
                    error: "Invalid credentials".to_string(),
                }),
            ));
        }
    };

    let valid = verify_password(&payload.password, &user.password_hash)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "Failed to verify password".to_string(),
                }),
            )
        })?;

    if !valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                error: "Invalid credentials".to_string(),
            }),
        ));
    }

    let secret = std::env::var("JWT_SECRET")
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: "JWT_SECRET is not configured".to_string(),
            }),
        )
    })?;

let token = create_token(user.id, &secret)
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: "Failed to create token".to_string(),
            }),
        )
    })?;

Ok(Json(LoginResponse { token }))
}

pub fn create_app(pool: AppState) -> Router {
    Router::new()
        .route("/", get(|| async { "Hello from Rust!" }))
       .route("/users", get(get_users).post(create_user))
       .route("/login", axum::routing::post(login_user))
        .route("/users/{id}", get(get_user).delete(delete_user).put(update_user))
        .route("/protected", get(protected_route))
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
    Json(payload): Json<UpdateUser>,
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

async fn protected_route(headers: HeaderMap) -> Result<String, StatusCode> {
    let authorization = headers.get("authorization");

    let token = authorization
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));

    let secret = std::env::var("JWT_SECRET").unwrap();

   let claims = token
    .and_then(|token| verify_token(token, &secret).ok())
    .ok_or(StatusCode::UNAUTHORIZED)?;

Ok(format!("Claims: {:?}", claims))
}

pub type AppState = sqlx::SqlitePool;