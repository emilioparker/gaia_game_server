use tokio::{sync::mpsc::Sender, net::UdpSocket};
use crate::{hero::hero_command::{HeroCommand, HeroCommandInfo}, gaia_mpsc::GaiaSender};

pub async fn process(player_id:u16, channel_player_tx : &GaiaSender<HeroCommand>)
{
    let command = HeroCommand
    {
        player_id,
        info: HeroCommandInfo::Disconnect()
    };

    cli_log::info!("got a command {:?}", command);
    channel_player_tx.send(command).await.unwrap();
}