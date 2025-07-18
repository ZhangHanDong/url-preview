#![cfg(feature = "cache")]

use crate::Preview;
use dashmap::DashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;

#[derive(Clone)]
pub struct Cache {
    cache: Arc<DashMap<String, Preview>>,
}

impl Cache {
    pub fn new(capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(100).unwrap());
        Self {
            cache: Arc::new(DashMap::with_capacity(capacity.get())),
        }
    }

    pub async fn get(&self, key: &str) -> Option<Preview> {
        self.cache.get(key).map(|entry| entry.clone())
    }

    pub async fn set(&self, key: String, value: Preview) {
        self.cache.insert(key, value);
    }
}
