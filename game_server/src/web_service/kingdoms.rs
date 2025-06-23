use futures_util::StreamExt;
use hyper::{Response, Body};

use crate::{get_faction_code, kingdom::kingdom_entity::KingdomEntity, long_term_storage_service::db_kingdom::StoredKingdom, map::tetrahedron_id::TetrahedronId, web_service::create_response_builder};

use super::AppContext;



pub(crate) async fn handle_request_kingdoms(context: super::AppContext, _req: hyper::Request<hyper::Body>) -> Result<hyper::Body, String> 
{
    let mut binary_data = Vec::<u8>::new();
    let data_collection: mongodb::Collection<StoredKingdom> = context.db_client.database("game").collection::<StoredKingdom>("kingdoms");

    // Look up one document:
    let mut cursor = data_collection
    .find(
        bson::doc! {
                "world_id": context.storage_game_map.world_id,
        },
        None,
    ).await
    .unwrap();

    let mut towers_count = 0;

    while let Some(result) = cursor.next().await 
    {
        match result 
        {
            Ok(doc) => 
            {
                let kingdom_entity = KingdomEntity 
                {
                    object_id: doc.id,
                    version: doc.version,
                    tetrahedron_id: TetrahedronId::from_string(&doc.tetrahedron_id),
                    faction: get_faction_code(&doc.faction),
                };
                towers_count += 1;
                let data_in_bytes = kingdom_entity.to_bytes();
                binary_data.extend_from_slice(&data_in_bytes);
            },
            Err(error_details) => 
            {
                cli_log::info!("error getting kingdoms from db with {:?}", error_details);
            },
        }
    }
    cli_log::info!("----- kingdoms {}", towers_count);
    Ok(Body::from(binary_data))
}

pub async fn handle_temp_kingdoms_request(context: AppContext) -> Result<Body, String>
{
    cli_log::info!("request temp kingdoms");
    let mut binary_data = Vec::<u8>::new();
    let temp_kingdoms = context.temp_kingdoms.lock().await;
    let size = temp_kingdoms.0;
    cli_log::info!("request temp kingdoms {}", size);
    binary_data.extend_from_slice(&temp_kingdoms.1[..size]);

    cli_log::info!("sending data back");
    Ok(Body::from(binary_data))
}