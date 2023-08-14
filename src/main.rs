use rustle::{get_input_task, get_output_task, ReturnMsg};
use std::sync::{atomic::AtomicI32, Arc};
use structopt::StructOpt;
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket = Arc::new(UdpSocket::bind("127.0.0.1:8080").await?);
    let current_queue_size = Arc::new(AtomicI32::new(0));

    let (tx, rx) = tokio::sync::mpsc::channel::<ReturnMsg>(100);
    let socket_ref = socket.clone();

    let output_task = tokio::spawn(get_output_task(rx, socket_ref));

    let socket_ref = socket.clone();
    let input_task = tokio::spawn(get_input_task(socket_ref, current_queue_size, tx));

    tokio::select! {
        _ = output_task => {}
        _ = input_task => {}
    }

    Ok(())
}
