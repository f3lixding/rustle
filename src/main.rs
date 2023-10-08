use rustle::get_input_task;
use rustle::QueryService;
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::net::UdpSocket;

#[derive(StructOpt)]
struct Opt {
    #[structopt(default_value = "8080", short, long)]
    port: i32,

    #[structopt(default_value = "3", short, long)]
    maxqueuesize: i32,

    #[structopt(default_value = "2001:558:feed::1:53", short, long)]
    router_addr: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // need to pass port and maxqueuesize to task instantiation
    let Opt {
        port,
        maxqueuesize,
        router_addr,
    } = Opt::from_args();
    let addr = format!("[::]:{}", port);
    let socket = Arc::new(UdpSocket::bind(addr).await?);

    tokio::fs::create_dir_all("var/db").await?;
    tokio::fs::write("var/db/init.txt", "/something/something/").await?;

    let mut query_service = QueryService::new(PathBuf::from("var/db/init.txt"))
        .index_db()
        .await?
        .register_for_periodic_update()?;
    let update_task_handle = query_service.gib_update_task_handle().unwrap();
    let input_task = tokio::spawn(get_input_task(
        maxqueuesize,
        socket.clone(),
        router_addr,
        query_service,
    ));

    tokio::select! {
        _ = input_task => {
            println!("Input task exited")
        }
        _ = update_task_handle => {
            println!("Update task exited")
        }
    }

    Ok(())
}
