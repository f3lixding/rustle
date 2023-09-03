use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

mod query_service;
mod util;

use util::get_query_from_bytes;

pub struct ReturnMsg {
    content: String,
    addr: std::net::SocketAddr,
}

/// Provider for task that listens for external messages.
/// Upon receiving a message (which would be a UDP packet, because it's a DNS query), it spawns a
/// task to process the query.
///
/// Currently there is a maximum queue size.
pub async fn get_input_task(
    max_queue_size: i32,
    socket: Arc<UdpSocket>,
    tx: mpsc::Sender<ReturnMsg>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut buf = [0; 1024];
    let current_queue_size = Arc::new(AtomicI32::new(0));
    async move {
        loop {
            let (size, addr) = socket.recv_from(&mut buf).await?;
            let content = buf[..size].to_vec();

            let current_queue_size = current_queue_size.clone();
            let tx = tx.clone();

            if current_queue_size.load(Ordering::Relaxed) >= max_queue_size {
                println!("queue full please try again later");
                continue;
            }
            current_queue_size.fetch_add(1, Ordering::Relaxed);

            tokio::spawn(async move {
                println!("received content... processing");
                tokio::time::sleep(Duration::from_secs(5)).await;

                // call byte handler to decode message and run a query
                let query = get_query_from_bytes(&content).unwrap();
                println!("query: {:?}", query);

                if let Err(e) = tx
                    .send(ReturnMsg {
                        content: "no error".to_string(),
                        addr,
                    })
                    .await
                {
                    println!("message send failed {}", e);
                }

                current_queue_size.fetch_sub(1, Ordering::Relaxed);
            });
        }
        #[allow(unreachable_code)]
        Ok(())
    }
    .await
}

/// Provider for task that listens for internal messages.
/// Upon receiving a message, it sends a udp response back to the target port.
pub async fn get_output_task(
    mut rx: mpsc::Receiver<ReturnMsg>,
    socket: Arc<UdpSocket>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    async move {
        while let Some(msg) = rx.recv().await {
            let ReturnMsg { content, addr } = msg;
            if let Err(_) = socket.send_to(&content.as_bytes(), &addr).await {
                println!("message send failed");
            }
        }
        Ok(())
    }
    .await
}
