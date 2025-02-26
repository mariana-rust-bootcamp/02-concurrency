use core::fmt;
use std::{
    collections::HashMap,
    fmt::Display,
    sync::{Arc, RwLock},
};

use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub struct Metrics {
    data: Arc<RwLock<HashMap<String, i64>>>,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn inc(&self, key: impl Into<String>) -> Result<()> {
        let mut data = self.data.write().map_err(|e| anyhow!(e.to_string()))?;
        // 返回可变引用, 如果不存在则插入0再返回可变引用
        let count = data.entry(key.into()).or_insert(0);
        *count += 1;
        Ok(())
    }
    // 快照
    pub fn snapshot(&self) -> Result<HashMap<String, i64>> {
        Ok(self
            .data
            .read()
            .map_err(|e| anyhow!(e.to_string()))?
            .clone())
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Metrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = String::new();
        let data = self.data.read().map_err(|_e| fmt::Error)?;
        // RwLockReadGuard实现了Deref trait, 解引用时会返回内部的HashMap
        for (k, v) in data.iter() {
            res.push_str(&format!("{}: {}\n", k, v));
        }
        write!(f, "{}", res)
    }
}
