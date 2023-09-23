use std::marker::PhantomData;
use std::{collections::HashMap, path::PathBuf};
use tokio::sync::RwLock;

use super::dns_query_answer::*;
use super::dns_query_question::*;

// We shall enforce the state transition order as reflected by the structs' order below:
#[allow(dead_code)]
struct NotIndexed;
#[allow(dead_code)]
struct NotRegisteredForPeriodicUpdate;
#[allow(dead_code)]
struct Ready;

/// The main struct used for handling DNS requests.
/// It will take a path to the local db of a block list. This list will be periodically updated
/// and thus will be guarded behind a read write lock.
pub struct QueryService<State = NotIndexed> {
    db_file_path: PathBuf,
    nono_list: RwLock<HashMap<String, Vec<u8>>>,
    state: PhantomData<State>,
}

impl QueryService<NotIndexed> {
    pub fn new(db_file_path: PathBuf) -> Self {
        QueryService {
            db_file_path,
            nono_list: RwLock::new(HashMap::new()),
            state: PhantomData,
        }
    }

    async fn index_db(self) -> QueryService<NotRegisteredForPeriodicUpdate> {
        let QueryService {
            db_file_path,
            nono_list,
            ..
        } = self;

        {
            let mut _cache = &mut nono_list.write().await;
            _cache.insert("test".to_string(), vec![1, 2, 3]);
        }

        QueryService {
            db_file_path,
            nono_list,
            state: PhantomData,
        }
    }
}

impl Default for QueryService<NotIndexed> {
    fn default() -> Self {
        QueryService {
            db_file_path: get_default_db_path(),
            nono_list: RwLock::new(HashMap::new()),
            state: PhantomData,
        }
    }
}

fn get_default_db_path() -> PathBuf {
    PathBuf::from("./db/")
}

impl QueryService<Ready> {
    pub async fn process_query(
        &self,
        query: &DNSQueryQuestion<'_>,
    ) -> Result<DNSQueryAnswer, Box<dyn std::error::Error + Send + Sync>> {
        Ok(DNSQueryAnswerBuilder::default().build()?)
    }

    pub async fn update_db(&self, mut new_db: HashMap<String, Vec<u8>>) {
        let mut cache = self.nono_list.write().await;
        std::mem::swap(&mut *cache, &mut new_db);
    }
}

async fn test() {
    let query_service = QueryService::new(get_default_db_path());
    let query_service = query_service.index_db().await;
}
