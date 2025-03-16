use tokio::sync::mpsc::{error::SendError, Receiver, Sender};


pub fn channel<T>(buffer: usize, server_state_callback: fn(usize)) -> (GaiaSender<T>, Receiver<T>) 
{
    let (tx, rx ): (Sender<T>, Receiver<T>) = tokio::sync::mpsc::channel::<T>(buffer);

    let gaia_sender = GaiaSender
    {
        sender: tx,
        callback: server_state_callback,
    };
    return (gaia_sender, rx);
}

pub struct GaiaSender<T>
{
    pub sender : Sender<T>,
    pub callback: fn(usize)
}

impl<T> GaiaSender<T> {
    pub async fn send_data(self, data: T) -> Result<(), SendError<T>> 
    {
        let capacity = self.sender.capacity();
        (self.callback)(capacity);
        self.sender.send(data).await
    }
}
