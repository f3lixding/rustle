use std::{collections::HashMap, path::PathBuf};
use tokio::sync::RwLock;

use crate::util::{DNSQueryAnswer, DNSQueryAnswerBuilder, DNSQueryQuestion};

// replace these types with real types
#[derive(Default, Debug, derive_builder::Builder)]
#[builder(pattern = "owned")]
pub struct QueryService {
    #[builder(default = "get_default_db_path()")]
    db_file_path: PathBuf,
    cache: RwLock<HashMap<String, Vec<u8>>>,
}

fn get_default_db_path() -> PathBuf {
    PathBuf::from("./db/")
}

impl QueryService {
    pub async fn process_query(
        &self,
        query: &DNSQueryQuestion<'_>,
    ) -> Result<DNSQueryAnswer, Box<dyn std::error::Error + Send + Sync>> {
        Ok(DNSQueryAnswerBuilder::default().build()?)
    }

    pub async fn update_db(&self, mut new_db: HashMap<String, Vec<u8>>) {
        let mut cache = self.cache.write().await;
        std::mem::swap(&mut *cache, &mut new_db);
    }
}
