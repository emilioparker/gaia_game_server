
use crate::gaia_mpsc::GaiaSender;
use crate::map::tetrahedron_id::TetrahedronId;
use crate::hero::hero_command::HeroCommand;
use crate::tower::{TowerCommand, TowerCommandInfo};

pub async fn process_request(
    hero_channel_tx : &GaiaSender<HeroCommand>,
    tower_channel_tx : &GaiaSender<TowerCommand>,
    data : &[u8])
{
    cli_log::info!("---- enter or exit tower");
    let start = 1;
    let end = start + 8;
    let _player_session_id = u64::from_le_bytes(data[start..end].try_into().unwrap());
    let start = end;

    let end = start + 2;
    let player_id = u16::from_le_bytes(data[start..end].try_into().unwrap());
    let start = end;

    let end = start + 6;
    let mut buffer = [0u8;6];
    buffer.copy_from_slice(&data[start..end]);
    let tile_id = TetrahedronId::from_bytes(&buffer);
    let start = end;

    let end = start + 1;
    let faction = data[start];
    let start = end;

    let end = start + 1;
    let points = data[start];
    let start = end;

    hero_channel_tx.send(HeroCommand 
        {
            player_id,
            info: crate::hero::hero_command::HeroCommandInfo::ExitTower(tile_id.clone(), faction, points) 
        }).await.unwrap();

    let tower_action = TowerCommand
    {
        id: tile_id,
        info: TowerCommandInfo::AttackTower(player_id, 0, faction, points as u32, 0)
    };

    cli_log::info!("got a {:?}", tower_action);

    tower_channel_tx.send(tower_action).await.unwrap();
}