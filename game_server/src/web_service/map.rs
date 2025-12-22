use hyper::{Body, Response, http::Error};
use crate::{long_term_storage_service::db_region::StoredRegion, map::tetrahedron_id::TetrahedronId};
use super::{create_response_builder, AppContext};


// why would I grab it from the database if I can grab it from ram ?? LIke the temp regions.
pub async fn handle_region_request(context: AppContext, data : Vec<&str>) -> Result<Body, String> 
{
    let mut iterator = data.into_iter();
    let region_list = iterator.next();
    let regions = if let Some(regions_csv) = region_list 
    {
        cli_log::info!("{}", regions_csv);
        let data = regions_csv.split(",");
        let regions_ids : Vec<&str> = data.collect();
        let iterator : Vec<TetrahedronId> = regions_ids.into_iter().map(|id| TetrahedronId::from_string(id)).collect();
        iterator
    }
    else 
    {
        Vec::new()
    };

    let mut binary_data = Vec::<u8>::new();

    let size_bytes = u32::to_le_bytes(regions.len() as u32);
    binary_data.extend_from_slice(&size_bytes);

    for region_id in &regions 
    {
        if let Some(region_data) = context.storage_game_map.stored_regions.get(region_id)
        {
            let lock = region_data.lock().await;
            let size_bytes = u32::to_le_bytes(lock.len() as u32);
            binary_data.extend_from_slice(&size_bytes);
            binary_data.extend_from_slice(&lock);
        }
        else 
        {
            let size_bytes = u32::to_le_bytes(0);
            binary_data.extend_from_slice(&size_bytes);
        }
    
        // let data_collection: mongodb::Collection<StoredRegion> = context.db_client.database("game").collection::<StoredRegion>("regions");

        // // Look up one document:
        // let data_from_db: Option<StoredRegion> = data_collection
        // .find_one(
        //     bson::doc! 
        //     {
        //             "world_id": context.storage_game_map.world_id,
        //             "region_id": region_id.to_string()
        //     },
        //     None,
        // ).await
        // .unwrap();

        // if let Some(region_from_db) = data_from_db 
        // {
        //     cli_log::info!("region id {:?} with version {}", region_from_db.region_id, region_from_db.region_version);
        //     let region_data: Vec<u8> = match region_from_db.compressed_data 
        //     {
        //         bson::Bson::Binary(binary) => binary.bytes,
        //         _ => panic!("Expected Bson::Binary"),
        //     };
        //     stored_regions_data.push(region_data);
        // }
        // else 
        // {
        //     stored_regions_data.push(Vec::new());
        // }
    }

    // for region_data in &stored_regions_data
    // {
    //     let size_bytes = u32::to_le_bytes(region_data.len() as u32);
    //     binary_data.extend_from_slice(&size_bytes);
    // }

    // for region_data in &mut stored_regions_data
    // {
    //     binary_data.append(region_data);
    // }

    //[regions_len][len_1][data][len_2][data]..
    Ok(Body::from(binary_data))
}

pub async fn handle_temp_region_request(context: AppContext, data : Vec<&str>) -> Result<Body, String> 
{
    let mut iterator = data.into_iter();
    let region_list = iterator.next();

// this string might contain more than one region separated by semicolon
    let regions = if let Some(regions_csv) = region_list 
    {
        cli_log::info!("{}", regions_csv);
        let data = regions_csv.split(",");
        let regions_ids : Vec<&str> = data.collect();
        let iterator : Vec<TetrahedronId> = regions_ids.into_iter().map(|id| TetrahedronId::from_string(id)).collect();
        iterator
    }
    else 
    {
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

    Ok(Body::from(binary_data))
}

pub async fn handle_temp_mob_region_request(context: AppContext, data : Vec<&str>) -> Result<Body, String> 
{
    let mut iterator = data.into_iter();
    let region_list = iterator.next();

// this string might contain more than one region separated by semicolon
    let regions = if let Some(regions_csv) = region_list 
    {
        cli_log::info!("{}", regions_csv);
        let data = regions_csv.split(",");
        let regions_ids : Vec<&str> = data.collect();
        let iterator : Vec<TetrahedronId> = regions_ids.into_iter().map(|id| TetrahedronId::from_string(id)).collect();
        iterator
    }
    else 
    {
        Vec::new()
    };

    let mut binary_data = Vec::<u8>::new();
    for region_id in &regions 
    {
        let region_map = context.temp_mobs_regions.get(region_id).unwrap();
        let region_map_lock = region_map.lock().await;
        let size = region_map_lock.index;
        binary_data.extend_from_slice(&region_map_lock.buffer[..size]);

        cli_log::info!("mob region size {size}");
    }

    Ok(Body::from(binary_data))
}