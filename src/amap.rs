/// 专门针对原子类型的并发安全map, 操作系统提供了原子操作, 不需要加锁
/// 由于AtomicI64没有实现Clone, 所以需要手动实现Clone trait
use std::{
    collections::HashMap,
    fmt::Display,
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
};

use anyhow::{anyhow, Result};

#[derive(Debug)]
pub struct AmapMetrics {
    data: Arc<HashMap<&'static str, AtomicI64>>,
}

impl AmapMetrics {
    pub fn new(metrics_name: &[&'static str]) -> Self {
        let map = metrics_name
            .iter()
            .map(|&name| (name, AtomicI64::new(0)))
            .collect();
        Self {
            data: Arc::new(map),
        }
    }
    pub fn inc(&self, key: impl AsRef<str>) -> Result<()> {
        let key = key.as_ref();
        let counter = self
            .data
            .get(key)
            .ok_or_else(|| anyhow!("key {} not found", key))?;
        counter.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

impl Clone for AmapMetrics {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data), // 保证只增加引用计数, 不会增加锁的所有权
        }
    }
}

impl Display for AmapMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, value) in self.data.iter() {
            writeln!(f, "{}: {}", key, value.load(Ordering::Relaxed))?;
        }
        Ok(())
    }
}
