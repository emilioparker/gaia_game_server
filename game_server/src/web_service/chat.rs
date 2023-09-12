use futures_util::StreamExt;
use hyper::{Response, Body};


use crate::chat::chat_entry::CHAT_ENTRY_SIZE;

use super::AppContext;

pub async fn handle_chat_record_request(context: AppContext) -> Result<Response<Body>, hyper::http::Error>
{
    println!("request chat record");
    let mut binary_data = Vec::<u8>::new();
    let messages = context.old_messages.lock().await;

    binary_data.extend_from_slice(&messages.record);

    println!("sending data back");
    let response = Response::builder()
        .status(hyper::StatusCode::OK)
        .header("Content-Type", "application/octet-stream")
        .body(Body::from(binary_data))
        .expect("Failed to create response");
    Ok(response)
}