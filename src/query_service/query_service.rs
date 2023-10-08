use chrono::Local;
use std::collections::HashSet;
use std::io::ErrorKind;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use super::dns_query_answer::*;
use super::dns_query_question::*;
use super::response::Response;

// We shall enforce the state transition order as reflected by the structs' order below:
#[allow(dead_code)]
pub struct NotIndexed;
#[allow(dead_code)]
pub struct NotRegisteredForPeriodicUpdate;
#[allow(dead_code)]
pub struct Ready;

type UpdateHandleReturnType = Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// The main struct used for handling DNS requests.
/// It will take a path to the local db of a block list. This list will be periodically updated
/// and thus will be guarded behind a read write lock.
pub struct QueryService<State = NotIndexed> {
    db_file_path: PathBuf,
    nono_list: Arc<RwLock<HashSet<String>>>,
    update_handle: Option<tokio::task::JoinHandle<UpdateHandleReturnType>>,
    state: PhantomData<State>,
}

impl QueryService<NotIndexed> {
    pub fn new(db_file_path: PathBuf) -> Self {
        QueryService {
            db_file_path,
            nono_list: Arc::new(RwLock::new(HashSet::new())),
            update_handle: None,
            state: PhantomData,
        }
    }

    pub async fn index_db(
        self,
    ) -> Result<
        QueryService<NotRegisteredForPeriodicUpdate>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let QueryService {
            db_file_path,
            nono_list,
            update_handle,
            ..
        } = self;

        {
            let mut nono_list = nono_list.write().await;
            let db_file = tokio::fs::read(&db_file_path).await?;
            let content = String::from_utf8(db_file)?;
            for line in content.lines() {
                nono_list.insert(line.to_string());
            }
        }

        Ok(QueryService {
            db_file_path,
            nono_list,
            update_handle,
            state: PhantomData,
        })
    }
}

impl QueryService<NotRegisteredForPeriodicUpdate> {
    pub fn register_for_periodic_update(
        self,
    ) -> Result<QueryService<Ready>, Box<dyn std::error::Error + Send + Sync>> {
        let QueryService {
            db_file_path,
            nono_list,
            ..
        } = self;

        let nono_list_ref = nono_list.clone();
        let db_file_path_clone = db_file_path.clone();
        let update_handle = {
            let handle = tokio::task::spawn(async move {
                // TODO: log this instead
                println!("Spawning periodic update task");
                loop {
                    tokio::select! {
                        _ = tokio::time::sleep(std::time::Duration::from_secs(60 * 60 * 24 * 7)) => {}
                        // TODO: add a notify here to wake this task up from sleep upon
                        // We can accept the notify as an arg from new()
                    }
                    // TODO: add actual db file update task here
                    // Refresh once every week.
                    let mut nono_list: HashSet<String> = HashSet::new();

                    // Download it from easylist
                    let response =
                        reqwest::get("https://easylist.to/easylist/easylist.txt").await?;
                    let list_content = response.text().await?;

                    // Populate the new map with it
                    for line in list_content.lines() {
                        nono_list.insert(line.to_string());
                    }

                    // swap
                    {
                        let mut nono_list_write_ref = nono_list_ref.write().await;
                        std::mem::swap(&mut *nono_list_write_ref, &mut nono_list);
                    }
                    // TODO: log this
                    let time_now = Local::now().format("%y-%m-%d-%H:%M:%S");
                    println!("Block list update completed at {}", time_now);

                    // Replace the file for record keeping
                    let db_file_dir = db_file_path_clone.parent().ok_or(std::io::Error::new(
                        ErrorKind::NotFound,
                        "Block list file parent not found",
                    ))?;
                    let new_db_file_path = db_file_dir.join(format!("block_list_{}.txt", time_now));
                    let mut file = tokio::fs::File::create(&new_db_file_path).await?;
                    file.write_all(list_content.as_bytes()).await?;
                    // TODO: log this
                    println!("Block list file {} updated", time_now);
                }

                #[allow(unreachable_code)]
                Ok(())
            });

            Some(handle)
        };

        Ok(QueryService {
            db_file_path,
            nono_list,
            update_handle,
            state: PhantomData,
        })
    }
}

impl QueryService<Ready> {
    /// This is the main entry point for request processing.
    /// The request shall be read into a byte vector.
    pub async fn process_bytes(
        &self,
        input_bytes: &Vec<u8>,
    ) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        let query = DNSQueryQuestion::try_from(input_bytes)?;
        println!("Query: {:?}", query);
        Ok(Response::Miss(query.message_id))
    }

    pub fn gib_update_task_handle(
        &mut self,
    ) -> Option<JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>> {
        self.update_handle.take()
    }
}
