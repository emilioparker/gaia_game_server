use hyper::{Body, Response, http::Error};
use crate::{long_term_storage_service::db_region::StoredRegion, map::tetrahedron_id::TetrahedronId};
use super::AppContext;

pub async fn handle_region_request(context: AppContext, data : Vec<&str>) -> Result<Response<Body>, Error> {

    let mut iterator = data.into_iter();
    let region_list = iterator.next();
    let regions = if let Some(regions_csv) = region_list {
        println!("{}", regions_csv);
        let data = regions_csv.split(",");
        let regions_ids : Vec<&str> = data.collect();
        let iterator : Vec<TetrahedronId> = regions_ids.into_iter().map(|id| TetrahedronId::from_string(id)).collect();
        iterator
    } else {
        Vec::new()
    };

    let mut binary_data = Vec::<u8>::new();
    let mut stored_regions_data = Vec::<Vec<u8>>::new();
    for region_id in &regions 
    {
        let data_collection: mongodb::Collection<StoredRegion> = context.db_client.database("game").collection::<StoredRegion>("regions");

        // Look up one document:
        let data_from_db: Option<StoredRegion> = data_collection
        .find_one(
            bson::doc! {
                    "world_id": context.storage_game_map.world_id,
                    "region_id": region_id.to_string()
            },
            None,
        ).await
        .unwrap();

        if let Some(region_from_db) = data_from_db {
            println!("region id {:?} with version {}", region_from_db.region_id, region_from_db.region_version);
            let region_data: Vec<u8> = match region_from_db.compressed_data {
                bson::Bson::Binary(binary) => binary.bytes,
                _ => panic!("Expected Bson::Binary"),
            };
            stored_regions_data.push(region_data);
        }
        else {
            stored_regions_data.push(Vec::new());
        }
    }

    for region_data in &stored_regions_data{
        let size_bytes = u32::to_le_bytes(region_data.len() as u32);
        binary_data.extend_from_slice(&size_bytes);
    }

    for region_data in &mut stored_regions_data
    {
        binary_data.append(region_data);
    }

    let response = Response::builder()
        .status(hyper::StatusCode::OK)
        .header("Content-Type", "application/octet-stream")
        .body(Body::from(binary_data))
        .expect("Failed to create response");
    Ok(response)
}

pub async fn handle_temp_region_request(context: AppContext, data : Vec<&str>) -> Result<Response<Body>, Error> {

    let mut iterator = data.into_iter();
    let region_list = iterator.next();

// this string might contain more than one region separated by semicolon
    let regions = if let Some(regions_csv) = region_list {
        println!("{}", regions_csv);
        let data = regions_csv.split(",");
        let regions_ids : Vec<&str> = data.collect();
        let iterator : Vec<TetrahedronId> = regions_ids.into_iter().map(|id| TetrahedronId::from_string(id)).collect();
        iterator
    } else {
        Vec::new()
    };

    let mut binary_data = Vec::<u8>::new();
    for region_id in &regions 
    {
        let region_map = context.temp_regions.get(region_id).unwrap();
        let region_map_lock = region_map.lock().await;
        let size = region_map_lock.index;
        binary_data.extend_from_slice(&region_map_lock.buffer[..size]);
    }

    let response = Response::builder()
        .status(hyper::StatusCode::OK)
        .header("Content-Type", "application/octet-stream")
        .body(Body::from(binary_data))
        .expect("Failed to create response");
    Ok(response)
}