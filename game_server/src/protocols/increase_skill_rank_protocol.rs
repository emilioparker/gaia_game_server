
use crate::{hero::hero_command::{HeroCommand, HeroCommandInfo}, gaia_mpsc::GaiaSender};

pub async fn process(data : &[u8], channel_player_tx : &GaiaSender<HeroCommand>)
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
    let skill_id = data[start];

    let command = HeroCommand
    {
        player_id,
        info: HeroCommandInfo::IncreaseSkillRank(skill_id)
    };

    cli_log::info!("got a command {:?}", command);

    channel_player_tx.send(command).await.unwrap();
}
