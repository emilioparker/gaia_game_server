use futures_util::StreamExt;
use hyper::{Response, Body};

use crate::{get_faction_code, long_term_storage_service::db_tower::StoredTower, map::tetrahedron_id::TetrahedronId, tower::tower_entity::{DamageByFaction, TowerEntity}, web_service::create_response_builder};

use super::AppContext;



pub(crate) async fn handle_request_towers(context: super::AppContext, _req: hyper::Request<hyper::Body>) -> Result<hyper::Response<hyper::Body>, hyper::http::Error> 
{
    let mut binary_data = Vec::<u8>::new();
    let data_collection: mongodb::Collection<StoredTower> = context.db_client.database("game").collection::<StoredTower>("towers");

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
                let tower_entity = TowerEntity 
                {
                    object_id: doc.id,
                    version: doc.version,
                    tetrahedron_id: TetrahedronId::from_string(&doc.tetrahedron_id),
                    cooldown: doc.cooldown,
                    event_id: doc.event_id,
                    faction: get_faction_code(&doc.faction),
                    damage_received_in_event: doc.damage_received_in_event.into_iter().map(|d| DamageByFaction
                    {
                        event_id: d.event_id,
                        faction: get_faction_code(&d.faction),
                        amount: d.amount,
                    }).collect(),
                };
                towers_count += 1;
                let data_in_bytes = tower_entity.to_bytes();
                binary_data.extend_from_slice(&data_in_bytes);
            },
            Err(error_details) => 
            {
                cli_log::info!("error getting towers from db with {:?}", error_details);
            },
        }
    }
    cli_log::info!("----- towers {}", towers_count);

    let response = create_response_builder()
        .body(Body::from(binary_data))
        .expect("Failed to create response");
    Ok(response)
}

pub async fn handle_temp_tower_request(context: AppContext) -> Result<Response<Body>, hyper::http::Error>
{
    cli_log::info!("request temp towers");
    let mut binary_data = Vec::<u8>::new();
    let temp_towers = context.temp_towers.lock().await;
    let size = temp_towers.0;
    cli_log::info!("request temp towers {}", size);
    binary_data.extend_from_slice(&temp_towers.1[..size]);

    cli_log::info!("sending data back");
    let response = create_response_builder()
        .body(Body::from(binary_data))
        .expect("Failed to create response");
    Ok(response)
}