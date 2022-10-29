
use std::collections::HashSet;
use std::sync::Arc;
use crate::map::GameMap;
use crate::map::map_entity::{MapEntity};
use crate::map::tetrahedron_id::TetrahedronId;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver};

pub fn start_server(
    mut tile_changes_rx : Receiver<MapEntity>,
    map : GameMap
) {

    let modified_regions = HashSet::<TetrahedronId>::new();
    let modified_regions_reference = Arc::new(Mutex::new(modified_regions));

    let modified_regions_update_lock = modified_regions_reference.clone();
    let modified_regions_reader_lock = modified_regions_reference.clone();

    let map_reference = Arc::new(map);
    let map_reader = map_reference.clone();
    let map_updater = map_reference.clone();


    tokio::spawn(async move {
        loop {
            let message = tile_changes_rx.recv().await.unwrap();
            println!("got a tile changed {:?} ", message);
            let region_id = message.id.get_parent(7);

            let mut modified_regions = modified_regions_update_lock.lock().await;
            modified_regions.insert(region_id.clone());

            let region = map_updater.get_region(&region_id);
            let mut locked_tiles = region.lock().await;

            let old = locked_tiles.get(&message.id);
            match old {
                Some(_previous_record) => {
                    locked_tiles.insert(message.id.clone(), message);
                }
                _ => {
                   locked_tiles.insert(message.id.clone(), message);
                }
            }
            // we need to save into the hashmap and then save to a file.
        }
    });

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(100)).await;
            let mut modified_regions = modified_regions_reader_lock.lock().await;

            for region_id in modified_regions.iter(){
                println!("this region was changed {}", region_id.to_string());
                let region = map_reader.get_region(region_id);
                let file_name = format!("map_initial_data/world001_{}_props.bytes", region_id.to_string());
                let mut file = File::create(file_name).await.unwrap();
                let locked_tiles = region.lock().await;
                for tile in locked_tiles.iter()
                {
                    let bytes = tile.1.to_bytes();
                    file.write_all(&bytes).await.unwrap();
                }

            }
            modified_regions.clear();
        }
    });
}



