use axum::http::version;
use tokio::{net::TcpListener, sync::{mpsc::{Receiver, Sender}, Mutex}};
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt, SinkExt}; // for reading/writing messages
use std::{collections::{vec_deque, HashMap}, net::SocketAddr, sync::Arc};
use bytes::Bytes;

pub struct WebSocketConnection
{
    pub address : SocketAddr,
    pub tx_from_server_to_client : Sender<u32>,
    pub rx_from_client_to_server : Receiver<Bytes>,
}


pub async fn run(clients : Arc<Mutex<HashMap<String, WebSocketConnection>>>) 
{
    // Bind to a local TCP socket
    let addr = "0.0.0.0:11005";
    let listener = TcpListener::bind(&addr).await.expect("Can't bind");
    println!("WebSocket server running at ws://{}", addr);

    // Accept incoming connections
    while let Ok((stream, socket_addr)) = listener.accept().await 
    {
        tokio::spawn(handle_connection(stream, socket_addr));
    }
}

async fn handle_connection(stream: tokio::net::TcpStream, addr: SocketAddr) 
{
    println!("New connection from {}", addr);

    let (tx_from_server_to_client, rx_from_server_to_client ): (Sender<u32>, Receiver<u32>) = tokio::sync::mpsc::channel::<u32>(10);
    let (tx_from_client_to_server, rx_from_client_to_server ): (Sender<Bytes>, Receiver<Bytes>) = tokio::sync::mpsc::channel::<Bytes>(10);

    let connection = WebSocketConnection
    {
        address: addr,
        tx_from_server_to_client,
        rx_from_client_to_server,
    };

    // Perform the WebSocket handshake
    let ws_stream = accept_async(stream)
        .await
        .expect("WebSocket handshake failed");

    println!("WebSocket connection established: {}", addr);

    // Split into sender and receiver
    let (mut write, mut read) = ws_stream.split();

    // Echo messages back to the client
    while let Some(msg) = read.next().await 
    {
        match msg 
        {
            Ok(msg) => 
            {
                println!("Received a message from {}: {}", addr, msg);
                if msg.is_text() || msg.is_binary() 
                {

                    tx_from_client_to_server.send(msg.into_data());
                    // write.send(msg).await.expect("Failed to send message");
                }
            }
            Err(e) => 
            {
                eprintln!("Error processing connection: {}", e);
                break;
            }
        }
    }

    println!("Connection {} closed", addr);
}