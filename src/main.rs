use futures::{future::select_all, future::FutureExt};
use rustle::get_input_tasks;
use rustle::QueryService;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::net::UdpSocket;

#[derive(StructOpt)]
struct Opt {
    #[structopt(default_value = "8080", short, long)]
    port: i32,

    #[structopt(default_value = "2001:558:feed::1:53", short, long)]
    router_addr: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let Opt { port, router_addr } = Opt::from_args();

    let main_addr = format!("[::]:{}", port);
    let sub_addr = "[::]:0";

    tokio::fs::create_dir_all("var/db").await?;
    tokio::fs::write("var/db/init.txt", "/something/something/").await?;

    let mut query_service = QueryService::new(PathBuf::from("var/db/init.txt"))
        .index_db()
        .await?
        .register_for_periodic_update()?;
    let update_task_handle = query_service
        .gib_update_task_handle()
        .ok_or("Update task handle is None")?;

    let cpu_num = num_cpus::get();
    let query_service = Arc::new(query_service);
    let mut main_listener_tasks = Vec::new();
    let mut subrequest_tasks = Vec::new();

    println!("Starting here");
    for _ in 0..cpu_num {
        let (main_socket, sub_socket) = {
            let main_socket = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?;
            main_socket.set_reuse_port(true)?;
            main_socket.bind(&main_addr.parse::<SocketAddr>()?.into())?;

            let sub_socket = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?;
            sub_socket.set_reuse_port(true)?;
            sub_socket.bind(&sub_addr.parse::<SocketAddr>()?.into())?;
            (
                UdpSocket::from_std(main_socket.into())?,
                UdpSocket::from_std(sub_socket.into())?,
            )
        };

        let (main_listener_task, subrequest_task) = get_input_tasks(
            main_socket.into(),
            sub_socket.into(),
            &router_addr,
            query_service.clone(),
        )
        .await?;
        main_listener_tasks.push(main_listener_task);
        subrequest_tasks.push(subrequest_task);
    }

    let main_listener_tasks = select_all(main_listener_tasks).fuse();
    let subrequest_tasks = select_all(subrequest_tasks).fuse();

    tokio::select! {
        _ = main_listener_tasks => {}
        _ = subrequest_tasks => {}
        update_res = update_task_handle => {
            match update_res {
                Ok(_) => println!("Update task exited normally"),
                Err(e) => println!("Update task exited with error: {:?}", e),
            }
        }
    }

    Ok(())
}
