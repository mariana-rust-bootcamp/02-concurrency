use std::{collections::HashMap, fmt::Display, sync::Arc};

use anyhow::Result;
use dashmap::DashMap;

#[derive(Debug, Clone)]
pub struct CmapMetrics {
    data: Arc<DashMap<String, i64>>,
}

impl CmapMetrics {
    pub fn new() -> Self {
        Self {
            data: Arc::new(DashMap::new()),
        }
    }

    pub fn inc(&self, key: impl Into<String>) -> Result<()> {
        // 使用dashmap就不需要手动加锁lock()/write()了
        // 返回可变引用, 如果不存在则插入0再返回可变引用
        let mut count = self.data.entry(key.into()).or_insert(0);
        *count += 1;
        Ok(())
    }
    // 快照 dashmap不允许直接获取内部的HashMap, 只能通过迭代器获取
    pub fn snapshot(&self) -> HashMap<String, i64> {
        let mut map = HashMap::new();
        for entry in self.data.iter() {
            map.insert(entry.key().to_string(), *entry.value());
        }
        map
    }
}

impl Default for CmapMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for CmapMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = String::new();
        // RwLockReadGuard实现了Deref trait, 解引用时会返回内部的HashMap
        for entry in self.data.iter() {
            res.push_str(&format!("{}: {}\n", entry.key(), entry.value()));
        }
        write!(f, "{}", res)
    }
}
