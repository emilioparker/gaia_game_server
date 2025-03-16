use tokio::{sync::mpsc::Sender, net::UdpSocket};

use crate::{gaia_mpsc::GaiaSender, map::tetrahedron_id::TetrahedronId, tower::{TowerCommand, TowerCommandInfo}};


pub async fn process(
     data : &[u8; 508],
    channel_tower_tx : &GaiaSender<TowerCommand>)
{
        let mut start = 1;
        let mut end = start + 8;
        let _player_session_id = u64::from_le_bytes(data[start..end].try_into().unwrap());

        start = end;
        end = start + 2;
        let player_id = u16::from_le_bytes(data[start..end].try_into().unwrap());

        start = end;
        end = start + 1;
        let _faction = data[start];
        start = end;

        end = start + 6;
        let mut buffer = [0u8;6];
        buffer.copy_from_slice(&data[start..end]);
        let tile_id = TetrahedronId::from_bytes(&buffer);

        start = end;
        end = start + 2;
        let repair_amount = u16::from_le_bytes(data[start..end].try_into().unwrap()); 
        // start = end;

        let tower_action = TowerCommand{
            id: tile_id,
            info: TowerCommandInfo::RepairTower(player_id, repair_amount)
        };

        cli_log::info!("got a {:?}", tower_action);

        channel_tower_tx.send(tower_action).await.unwrap();
}