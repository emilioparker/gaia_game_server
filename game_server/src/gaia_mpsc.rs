use std::sync::Arc;

use tokio::sync::mpsc::{error::SendError, Receiver, Sender};

use crate::{ServerChannels, ServerState};



pub fn channel<T>(buffer: usize, channel : ServerChannels, server_state: Arc<ServerState>) -> (GaiaSender<T>, Receiver<T>) 
{
    let (tx, rx ): (Sender<T>, Receiver<T>) = tokio::sync::mpsc::channel::<T>(buffer);

    let capacity = tx.capacity();
    server_state.channels[&channel].store(capacity as u16, std::sync::atomic::Ordering::Relaxed);

    let gaia_sender = GaiaSender
    {
        sender: tx,
        channel,
        server_state
    };
    return (gaia_sender, rx);
}

pub struct GaiaSender<T>
{
    pub sender : Sender<T>,
    pub channel: ServerChannels,
    pub server_state: Arc<ServerState>,
}

impl<T> GaiaSender<T> 
{
    pub async fn send(&self, data: T) -> Result<(), SendError<T>> 
    {
        let capacity = self.sender.capacity();
        self.server_state.channels[&self.channel].store(capacity as u16, std::sync::atomic::Ordering::Relaxed);
        self.sender.send(data).await
    }
}

impl<T> Clone for GaiaSender<T> {
    fn clone(&self) -> Self 
    {
        Self 
        {
            sender: self.sender.clone(), // Sender<T> is cloneable
            channel: self.channel.clone(), // ServerChannels must be cloneable
            server_state: self.server_state.clone(), // Clone Arc (increases ref count)
        }
    }
}
