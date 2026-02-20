use axum::extract::Request;
use axum::http::header;
use axum::http::Method;
use axum::http::StatusCode;
use axum::middleware;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::Router;

use tower_http::services::ServeDir;
use tower_http::cors::CorsLayer;
use tower_http::cors::Any;

// pub mod compression_layer;

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
        .layer(middleware::from_fn(print_request_response))
        .layer(cors);
        // .layer(map_response(set_header));

    cli_log::info!("Server running at http://localhost:3030");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

async fn print_request_response(
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> 
{
    let path = req.uri().path().to_owned();
    let mut res = next.run(req).await;

    if path.contains("wasm")
    {
        res.headers_mut().insert("Content-Type", "application/wasm".parse().unwrap());
    }

    if path.contains(".gz")
    {
        res.headers_mut().insert("Content-Encoding", "gzip".parse().unwrap());
    }
    Ok(res)
}