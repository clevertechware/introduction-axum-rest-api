use axum::Router;
use axum::routing::get;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // initialize tracing for logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    dotenv().ok();
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .connect(&url)
        .await
        .expect("Failed to connect to database");
    info!("Connected to the database!");

    // build our application with a route
    let app = Router::new()
        // Get '/' goes to root
        .route("/", get(root))
        // Extension layer
        .layer(axum::Extension(pool));

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
