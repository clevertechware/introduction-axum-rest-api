use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use axum_jwt_oidc::{OidcAuthLayer, OidcConfig, OidcValidator, Validation};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::fmt::{Display, Formatter};
use tracing::{debug, info};

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // initialize tracing for logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    dotenv().ok();
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .connect(&url)
        .await
        .expect("Failed to connect to database");
    info!("Connected to the database!");

    // Initialize OIDC validator
    let config = OidcConfig::new(
        "http://your-oidc-provider.com".to_string(),
        "your-client-id".to_string(),
        "https://your-oidc-provider.com/.well-known/jwks.json".to_string(),
    );
    let oidc_validator = OidcValidator::new(config);

    // Configure validation rules
    let validation = Validation::default();
    // Create the authentication layer
    let auth_layer = OidcAuthLayer::<CustomClaims>::new(oidc_validator, validation);

    // build our application with a route
    let app = Router::new()
        // Get '/' goes to root
        .route("/", get(root))
        .route("/authors", post(create_author))
        .route("/posts", get(get_posts).post(create_post))
        .route(
            "/posts/{id}",
            get(get_post).put(update_post).delete(delete_post),
        )
        // Extension layer
        .layer(Extension(pool));

    // run our app with hyper, listening globally on port 5000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8888").await.unwrap();
    info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

// handler for root
async fn root() -> &'static str {
    "Hello, World!"
}

#[derive(Serialize, Deserialize)]
struct Post {
    id: i32,
    author_id: Option<i32>,
    title: String,
    body: String,
}

async fn get_posts(
    Extension(claims): Extension<CustomClaims>,
    Extension(pool): Extension<Pool<Postgres>>,
) -> Result<Json<Vec<Post>>, StatusCode> {
    let posts = sqlx::query_as!(Post, "SELECT id, author_id, title, body FROM posts")
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    debug!("Posts fetched by {}", claims.sub);
    Ok(Json(posts))
}

async fn get_post(
    Extension(pool): Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
) -> Result<Json<Post>, StatusCode> {
    let post = sqlx::query_as!(
        Post,
        "SELECT id, author_id, title, body FROM posts WHERE id = $1",
        id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(post))
}

#[derive(Serialize, Deserialize)]
struct CreatePost {
    title: String,
    body: String,
    author_id: Option<i32>,
}

async fn create_post(
    Extension(claims): Extension<CustomClaims>,
    Extension(pool): Extension<Pool<Postgres>>,
    Json(new_post): Json<CreatePost>,
) -> Result<Json<Post>, StatusCode> {
    let post = sqlx::query_as!(
        Post,
        "INSERT INTO posts (author_id, title, body) VALUES ($1, $2, $3) RETURNING id, title, body, author_id",
        new_post.author_id,
        new_post.title,
        new_post.body
    )
        .fetch_one(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    debug!("Post created by {}", claims.sub);

    Ok(Json(post))
}

#[derive(Serialize, Deserialize)]
struct UpdatePost {
    title: String,
    body: String,
    author_id: Option<i32>,
}

async fn update_post(
    Extension(pool): Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
    Json(updated_post): Json<UpdatePost>,
) -> Result<Json<Post>, StatusCode> {
    let post = sqlx::query_as!(
        Post,
        "UPDATE posts SET title = $1, body = $2, author_id = $3 WHERE id = $4 RETURNING id, author_id, title, body",
        updated_post.title,
        updated_post.body,
        updated_post.author_id,
        id
    )
        .fetch_one(&pool)
        .await;

    match post {
        Ok(post) => Ok(Json(post)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn delete_post(
    Extension(pool): Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let result = sqlx::query!("DELETE FROM posts WHERE id = $1", id)
        .execute(&pool)
        .await;

    match result {
        Ok(_) => Ok(Json(serde_json::json!({
            "message": "Post deleted successfully"
        }))),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Serialize, Deserialize)]
struct CreateAuthor {
    name: String,
}

impl Display for CreateAuthor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "CreateAuthor(name={})", self.name)
    }
}

#[derive(Serialize, Deserialize)]
struct Author {
    id: i32,
    name: String,
}

async fn create_author(
    Extension(claims): Extension<CustomClaims>,
    Extension(pool): Extension<Pool<Postgres>>,
    Json(new_author): Json<CreateAuthor>,
) -> Result<Json<Author>, StatusCode> {
    let author = sqlx::query_as!(
        Author,
        "INSERT INTO authors (name) VALUES ($1) RETURNING id, name",
        new_author.name,
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    info!("Author created: {} by {}", new_author, claims.sub);
    Ok(Json(author))
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CustomClaims {
    sub: String,
    email: Option<String>,
    // Add your custom claims here
}
