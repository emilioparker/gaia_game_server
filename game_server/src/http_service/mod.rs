use axum::{
    http::{header, HeaderValue, Method},
    response::{IntoResponse, Response},
    Router,
};
use tower_http::{
    services::ServeDir,
    cors::{CorsLayer, Any},
};

pub async fn run() 
{
    // Serve static files from the `public` folder
    let file_service = ServeDir::new("public")
        .not_found_service(ServeDir::new("public")); // Fallback for 404s (e.g. index.html for SPA)

    // CORS setup: allow all origins, methods, and headers
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::ACCEPT]);

    // Build Axum app
    let app = Router::new()
        .fallback_service(file_service)
        .layer(cors);

    cli_log::info!("Server running at http://localhost:8000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}