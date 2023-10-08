use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use tokio::net::UdpSocket;

mod query_service;

pub use query_service::{QueryService, Ready};

/// Provider for task that listens for external messages.
/// Upon receiving a message (which would be a UDP packet, because it's a DNS query), it spawns a
/// task to process the query.
///
/// Currently there is a maximum queue size.
pub async fn get_input_task(
    max_queue_size: i32,
    socket: Arc<UdpSocket>,
    router_addr: String,
    query_service: QueryService<Ready>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut buf = [0; 1024];
    let current_queue_size = Arc::new(AtomicI32::new(0));

    async move {
        let query_service = Arc::new(query_service);
        let router_addr = Arc::new(router_addr);
        loop {
            let (size, addr) = socket.recv_from(&mut buf).await?;
            let content = buf[..size].to_vec();

            let current_queue_size = current_queue_size.clone();

            if current_queue_size.load(Ordering::Relaxed) >= max_queue_size {
                println!("queue full please try again later");
                continue;
            }

            let query_service = query_service.clone();
            let socket = socket.clone();
            let router_addr = router_addr.clone();
            current_queue_size.fetch_add(1, Ordering::Relaxed);

            tokio::spawn(async move {
                // TODO: log this
                println!("received content... processing");

                // call byte handler to decode message and run a query
                let _ = query_service.process_bytes(&content).await;
                _ = socket.send_to(&content, router_addr.as_str()).await?;

                current_queue_size.fetch_sub(1, Ordering::Relaxed);
                Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
            });
        }
        #[allow(unreachable_code)]
        Ok(())
    }
    .await
}
