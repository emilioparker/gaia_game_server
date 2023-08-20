use tokio::{sync::mpsc::Sender, net::UdpSocket};

use crate::{tower::{TowerCommand, TowerCommandInfo}, map::tetrahedron_id::TetrahedronId};


pub async fn process(
    _socket:&UdpSocket,
     data : &[u8; 508],
    channel_tower_tx : &Sender<TowerCommand>)
{
        let mut start : usize;
        let mut end : usize;

        start = 1; // ignoring first byte
        end = start + 6;
        let mut buffer = [0u8;6];
        buffer.copy_from_slice(&data[start..end]);
        let tile_id = TetrahedronId::from_bytes(&buffer);
        start = end;

        end = start + 2;
        let player_id = u16::from_le_bytes(data[start..end].try_into().unwrap()); 
        start = end;

        end = start + 1;
        let faction = data[start]; 
        start = end;

        end = start + 2;
        let damage = u16::from_le_bytes(data[start..end].try_into().unwrap()); 
        start = end;

        end = start + 4;
        let required_time = u32::from_le_bytes(data[start..end].try_into().unwrap()); 
        // start = end;

        let tower_action = TowerCommand{
            id: tile_id,
            info: TowerCommandInfo::AttackTower(player_id, faction, damage, required_time)
        };

        // println!("got a {:?}", map_action);

        channel_tower_tx.send(tower_action).await.unwrap();
}