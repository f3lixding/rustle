use rustle::{get_input_task, get_output_task, ReturnMsg};
use std::sync::Arc;
use structopt::StructOpt;
use tokio::net::UdpSocket;

#[derive(StructOpt)]
struct Opt {
    #[structopt(default_value = "8080", short, long)]
    port: i32,

    #[structopt(default_value = "3", short, long)]
    maxqueuesize: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // need to pass port and maxqueuesize to task instantiation
    let Opt { port, maxqueuesize } = Opt::from_args();
    let addr = format!("127.0.0.1:{}", port);
    let socket = Arc::new(UdpSocket::bind(addr).await?);
    let (tx, rx) = tokio::sync::mpsc::channel::<ReturnMsg>(100);

    let socket_ref = socket.clone();
    let output_task = tokio::spawn(get_output_task(rx, socket_ref));

    let socket_ref = socket.clone();
    let input_task = tokio::spawn(get_input_task(maxqueuesize, socket_ref, tx));

    tokio::select! {
        _ = output_task => {
            println!("Output task exited")
        }
        _ = input_task => {
            println!("Input task exited")
        }
    }

    Ok(())
}
