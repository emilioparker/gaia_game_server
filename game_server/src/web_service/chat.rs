use hyper::{Response, Body};


use super::AppContext;

pub async fn handle_chat_record_request(context: AppContext, data : Vec<&str>) -> Result<Response<Body>, hyper::http::Error>
{
    let mut iterator = data.into_iter();
    let message_faction = if let Some(faction) = iterator.next()
    {
        let result = faction.parse::<i32>();
        result.unwrap_or(0)
    }
    else
    {
        0
    };

    println!("request chat record");
    let mut binary_data = Vec::<u8>::new();
    let record = context.old_messages.lock().await;
    if let Some(messages) = record.get(&(message_faction as u8))
    {
        binary_data.extend_from_slice(&messages.record);
    }
    if let Some(global_messages) = record.get(&0)
    {
        binary_data.extend_from_slice(&global_messages.record);
    }

    println!("sending data back");
    let response = Response::builder()
        .status(hyper::StatusCode::OK)
        .header("Content-Type", "application/octet-stream")
        .body(Body::from(binary_data))
        .expect("Failed to create response");
    Ok(response)
}