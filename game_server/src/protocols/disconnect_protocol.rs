use tokio::{sync::mpsc::Sender, net::UdpSocket};
use crate::character::character_command::{CharacterCommand, CharacterCommandInfo};

pub async fn process(player_id:u16, channel_player_tx : &Sender<CharacterCommand>)
{
    let command = CharacterCommand
    {
        player_id,
        info: CharacterCommandInfo::Disconnect()
    };

    cli_log::info!("got a command {:?}", command);
    channel_player_tx.send(command).await.unwrap();
}