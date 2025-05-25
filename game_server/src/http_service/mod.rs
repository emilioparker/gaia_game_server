use axum::{
    body::Body, extract::{Request, State}, http::{header, HeaderValue, Method, StatusCode}, middleware::{self, map_response, map_response_with_state, Next}, response::{IntoResponse, Response}, Router
};

use tower_http::{
    services::ServeDir,
    cors::{CorsLayer, Any},
};

// pub mod compression_layer;

pub async fn run() 
{
    // Serve static files from the `public` folder
    let file_service = ServeDir::new("public")
    // .precompressed_gzip()
        .not_found_service(ServeDir::new("public")); // Fallback for 404s (e.g. index.html for SPA)

    // CORS setup: allow all origins, methods, and headers
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::ACCEPT]);

    // async fn set_header<B>(mut response: Response<B>) -> Response<B> 
    // {
    //     response.headers_mut().insert("x-foo", "foo".parse().unwrap());
    //     response
    // }


    // Build Axum app
    let app = Router::new()
        .fallback_service(file_service)
        .layer(middleware::from_fn(print_request_response))
        .layer(cors);
        // .layer(map_response(set_header));

    cli_log::info!("Server running at http://localhost:8000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

async fn print_request_response(
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> 
{
    // let (parts, body) = req.uri();
    let path = req.uri().path().to_owned();
    // // let bytes = buffer_and_print("request", body).await?;
    // // let req = Request::from_parts(parts, Body::from(bytes));
    // cli_log::info!("--------got a request for file {}", path);

// 13:24:55.365 [INFO] game_server::http_service: --------got a request for file /Build/public.loader.js
// 13:24:55.376 [INFO] game_server::http_service: --------got a request for file /Build/public.framework.js.gz
// 13:24:55.391 [INFO] game_server::http_service: --------got a request for file /Build/public.wasm.gz
// 13:24:55.422 [INFO] game_server::http_service: --------got a request for file /Build/public.wasm.gz
// 13:24:55.732 [INFO] game_server::http_service: --------got a request for file /Build/public.data.gz


//     AddEncoding br .unityweb
//   AddEncoding br .wasm
//   AddType application/wasm .wasm

    let mut res = next.run(req).await;

    if path.contains("wasm")
    {
        res.headers_mut().insert("Content-Type", "application/wasm".parse().unwrap());
        // cli_log::info!("it is a wasm we should ad dthe tontent type ");
    }

    if path.contains(".gz")
    {
        res.headers_mut().insert("Content-Encoding", "gzip".parse().unwrap());
        // cli_log::info!("it is a gz file, we add enconding");
    }

    // let (parts, body) = res.into_parts();
    // let bytes = buffer_and_print("response", body).await?;
    // let res = Response::from_parts(parts, Body::from(bytes));

    // Content-Encoding: gzip
    // Err((StatusCode::BAD_REQUEST, "failed".to_owned()))
    Ok(res)
}