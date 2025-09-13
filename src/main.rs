use async_oidc_jwt_validator::Algorithm;
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
async fn main() {
    // initialize tracing for logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");
    info!("Connected to the database!");

    // Exécution des migrations
    sqlx::migrate!("./migrations").run(&pool).await.expect("Failed to run migrations");
    info!("Migrations executed successfully");

    // Initialize OIDC validator
    let config = OidcConfig::new_with_discovery(
        std::env::var("ISSUER_URL").expect("ISSUER_URL must be set"),
        "".to_string(),
    )
    .await
    .unwrap();
    let oidc_validator = OidcValidator::new(config);
    // Configure validation rules
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_aud = false;
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
        .layer(Extension(pool))
        .layer(auth_layer);

    // run our app with hyper, listening globally on port 8000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await.expect("Failed to bind on port 8000");
    info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
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
    claims: Option<Extension<CustomClaims>>,
    Extension(pool): Extension<Pool<Postgres>>,
) -> Result<Json<Vec<Post>>, StatusCode> {
    if let Some(Extension(claims)) = claims {
        let posts = sqlx::query_as!(Post, "SELECT id, author_id, title, body FROM posts")
            .fetch_all(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        debug!("Posts fetched by {}", claims.sub);
        Ok(Json(posts))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn get_post(
    claims: Option<Extension<CustomClaims>>,
    Extension(pool): Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
) -> Result<Json<Post>, StatusCode> {
    if let Some(Extension(_)) = claims {
        let post = sqlx::query_as!(
            Post,
            "SELECT id, author_id, title, body FROM posts WHERE id = $1",
            id
        )
        .fetch_one(&pool)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

        Ok(Json(post))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

#[derive(Serialize, Deserialize)]
struct CreatePost {
    title: String,
    body: String,
    author_id: Option<i32>,
}

async fn create_post(
    claims: Option<Extension<CustomClaims>>,
    Extension(pool): Extension<Pool<Postgres>>,
    Json(new_post): Json<CreatePost>,
) -> Result<Json<Post>, StatusCode> {
    if let Some(Extension(claims)) = claims {
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
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

#[derive(Serialize, Deserialize)]
struct UpdatePost {
    title: String,
    body: String,
    author_id: Option<i32>,
}

async fn update_post(
    claims: Option<Extension<CustomClaims>>,
    Extension(pool): Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
    Json(updated_post): Json<UpdatePost>,
) -> Result<Json<Post>, StatusCode> {
    if let Some(Extension(_)) = claims {
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
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn delete_post(
    claims: Option<Extension<CustomClaims>>,
    Extension(pool): Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(Extension(_)) = claims {
        let result = sqlx::query!("DELETE FROM posts WHERE id = $1", id)
            .execute(&pool)
            .await;

        match result {
            Ok(_) => Ok(Json(
                serde_json::json!({"message": "Post deleted successfully"}),
            )),
            Err(_) => Err(StatusCode::NOT_FOUND),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
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
    claims: Option<Extension<CustomClaims>>,
    Extension(pool): Extension<Pool<Postgres>>,
    Json(new_author): Json<CreateAuthor>,
) -> Result<Json<Author>, StatusCode> {
    if let Some(Extension(claims)) = claims {
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
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CustomClaims {
    sub: String,
    email: Option<String>,
    // Add your custom claims here
}
