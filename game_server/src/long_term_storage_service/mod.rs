
use std::sync::Arc;
use std::{collections::HashMap};
use crate::map::map_entity::{MapEntity, MapCommand};
use crate::map::tetrahedron_id::TetrahedronId;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};

pub fn start_server(
    mut tile_changes_rx : Receiver<MapEntity>,
) {
    let all_tiles = HashMap::<TetrahedronId,MapEntity>::new();
    let tiles_update_mutex = Arc::new(Mutex::new(all_tiles));
    let tiles_save_mutex = tiles_update_mutex.clone();

    tokio::spawn(async move {
        loop {
            let message = tile_changes_rx.recv().await.unwrap();
            println!("got a tile changed {:?} ", message);
            let mut locked_tiles = tiles_update_mutex.lock().await;

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
            tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
            let locked_tiles = tiles_save_mutex.lock().await;

            let mut file = File::create("map_1.tiles").await.unwrap();
            for tile in locked_tiles.iter()
            {
                let bytes = tile.1.to_bytes();
                file.write_all(&bytes).await.unwrap();
            }
        }
    });
}


