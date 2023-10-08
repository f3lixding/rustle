use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

mod query_service;

pub use query_service::{QueryService, Ready, Response};

type LongRunningTaskType = JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>;

/// Provider for task that listens for external messages.
/// Upon receiving a message (which would be a UDP packet, because it's a DNS query), it spawns a
/// task to process the query.
///
/// Currently there is a maximum queue size.
pub async fn get_input_tasks(
    max_queue_size: i32,
    socket_orig_sender: Arc<UdpSocket>,
    router_addr: String,
    query_service: QueryService<Ready>,
) -> Result<(LongRunningTaskType, LongRunningTaskType), Box<dyn std::error::Error + Send + Sync>> {
    // 4 kbytes of buffer allocated.
    // This should be fine since historically DNS queries operated within the UDP packet limit,
    // which is only 512 bytes
    let mut buf = [0; 1024];
    let current_queue_size = Arc::new(AtomicI32::new(0));
    let parking_lot: Arc<RwLock<HashMap<u16, SocketAddr>>> = Arc::new(RwLock::new(HashMap::new()));
    let parking_lot_clone = parking_lot.clone();
    // We'll use a different socket for now for subrequest. Technically we do not need to do that.
    let socket_subrequest = Arc::new(UdpSocket::bind("[::]:0").await?);
    let socket_subrequest_clone = socket_subrequest.clone();

    let main_listener_task = tokio::spawn(async move {
        let query_service = Arc::new(query_service);
        let router_addr = Arc::new(router_addr);
        loop {
            let (size, addr) = socket_orig_sender.recv_from(&mut buf).await?;
            let content = buf[..size].to_vec();

            let current_queue_size = current_queue_size.clone();

            if current_queue_size.load(Ordering::Relaxed) >= max_queue_size {
                println!("queue full please try again later");
                continue;
            }

            let socket_subrequest = socket_subrequest_clone.clone();
            let parking_lot = parking_lot_clone.clone();
            let query_service = query_service.clone();
            let socket_orig_sender = socket_orig_sender.clone();
            let router_addr = router_addr.clone();
            current_queue_size.fetch_add(1, Ordering::Relaxed);

            tokio::spawn(async move {
                // TODO: log this
                println!("received content... processing");

                // call byte handler to decode message and run a query
                match query_service.process_bytes(&content).await? {
                    Response::Hit(bytes) => {}
                    Response::Miss(id) => {
                        {
                            parking_lot.write().await.insert(id, addr);
                        }
                        _ = socket_subrequest
                            .send_to(&content, router_addr.as_str())
                            .await?;
                    }
                }

                current_queue_size.fetch_sub(1, Ordering::Relaxed);
                Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
            });
        }
        // TODO: Add shutdown routine
        #[allow(unreachable_code)]
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    });

    let subrequest_task = tokio::spawn(async move {
        let mut buf = [0; 1024];
        loop {
            let (size, addr) = socket_subrequest.recv_from(&mut buf).await?;
            println!("Reponse received from {}", addr);
        }
        // TODO: Add shutdown routine
        #[allow(unreachable_code)]
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    });

    Ok::<_, Box<dyn std::error::Error + Send + Sync>>((main_listener_task, subrequest_task))
}
