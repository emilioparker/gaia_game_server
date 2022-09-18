use tokio::net::UdpSocket;

use crate::client_handler::ClientAction;
use tokio::sync::mpsc::Sender;

pub fn decode_float(buffer: &[u8;508], start: &mut usize, end: &mut usize) -> f32
{
    let decoded_float = f32::from_le_bytes(buffer[*start..(*start + 4)].try_into().unwrap());
    *start = *end;
    *end = *start + 4;

    decoded_float
}

pub async fn process_movement(_socket:&UdpSocket, data : &[u8; 508], channel_tx : &Sender<ClientAction>)
{
    let mut start = 1;
    let mut end = start + 8;

    let player_id = u64::from_le_bytes(data[start..end].try_into().unwrap());
    start = end;
    end = start + 4;

    let pos_x = decode_float(data, &mut start, &mut end);
    let pos_y = decode_float(data, &mut start, &mut end);
    let pos_z = decode_float(data, &mut start, &mut end);
    let position = [pos_x, pos_y, pos_z];

    let direction_x = decode_float(data, &mut start, &mut end);
    let direction_y = decode_float(data, &mut start, &mut end);
    let direction_z = decode_float(data, &mut start, &mut end);
    let direction = [direction_x, direction_y, direction_z];

    let action = u32::from_le_bytes(data[start..(start + 4)].try_into().unwrap());

    let client_action = ClientAction {
        player_id,
        position,
        direction,
        action
    };

    // println!("got a {:?}", position);

    channel_tx.send(client_action).await.unwrap();
}