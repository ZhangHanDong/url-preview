use crate::Preview;
use dashmap::DashMap;
use std::sync::Arc;
use std::num::NonZeroUsize;

#[derive(Clone)]
pub struct Cache {
    cache: Arc<DashMap<String, Preview>>,
}

impl Cache {
    pub fn new(capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(100).unwrap());
        Self {
            cache: Arc::new(DashMap::with_capacity(capacity.get()))
        }
    }

    pub async fn get(&self, key: &str) -> Option<Preview> {
        self.cache.get(key).map(|entry| entry.clone())
    }

    pub async fn set(&self, key: String, value: Preview) {
        self.cache.insert(key, value);
    }
}


// use crate::Preview;
// use lru::LruCache;
// use std::num::NonZeroUsize;
// use std::sync::Arc;
// use tokio::sync::Mutex;

// #[derive(Clone)]
// pub struct Cache {
//     cache: Arc<Mutex<LruCache<String, Preview>>>,
// }

// impl Cache {
//     pub fn new(capacity: usize) -> Self {
//         let capacity = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(100).unwrap());
//         Self {
//             cache: Arc::new(Mutex::new(LruCache::new(capacity))),
//         }
//     }

//     pub async fn get(&self, key: &str) -> Option<Preview> {
//         let mut cache = self.cache.lock().await;
//         cache.get(key).cloned()
//     }

//     pub async fn set(&self, key: String, value: Preview) {
//         let mut cache = self.cache.lock().await;
//         cache.put(key, value);
//     }
// }
