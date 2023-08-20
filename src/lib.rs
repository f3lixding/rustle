use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

mod byte_handler;

use byte_handler::decode_some_bytes;

pub struct ReturnMsg {
    content: String,
    addr: std::net::SocketAddr,
}

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
                decode_some_bytes(&content).unwrap();

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
