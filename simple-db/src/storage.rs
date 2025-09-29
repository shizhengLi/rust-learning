use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::error::{DatabaseError, Result};
use crate::types::{Table, Value, Row, Schema};

/// 存储操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageOperation {
    Create { table: String, schema: Schema },
    Insert { table: String, row: Row },
    Update { table: String, id: String, data: Vec<(String, Value)> },
    Delete { table: String, id: String },
    Drop { table: String },
}

/// 事务日志条目
#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: u64,
    pub timestamp: DateTime<Utc>,
    pub operation: StorageOperation,
}

impl LogEntry {
    pub fn new(id: u64, operation: StorageOperation) -> Self {
        Self {
            id,
            timestamp: Utc::now(),
            operation,
        }
    }
}

/// 持久化快照
#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub tables: Vec<Table>,
    pub timestamp: DateTime<Utc>,
    pub last_log_id: u64,
}

impl Snapshot {
    pub fn new(tables: Vec<Table>, last_log_id: u64) -> Self {
        Self {
            tables,
            timestamp: Utc::now(),
            last_log_id,
        }
    }
}

/// 存储引擎
pub struct StorageEngine {
    data_dir: String,
    log_file: String,
    snapshot_file: String,
    current_log_id: u64,
}

impl StorageEngine {
    /// 创建新的存储引擎
    pub fn new() -> Self {
        let data_dir = "data".to_string();
        let log_file = format!("{}/transaction.log", data_dir);
        let snapshot_file = format!("{}/snapshot.json", data_dir);

        Self {
            data_dir,
            log_file,
            snapshot_file,
            current_log_id: 0,
        }
    }

    /// 初始化存储目录
    pub fn initialize(&self) -> Result<()> {
        if !Path::new(&self.data_dir).exists() {
            fs::create_dir_all(&self.data_dir)?;
        }
        Ok(())
    }

    /// 写入日志
    pub fn write_log(&mut self, operation: StorageOperation) -> Result<()> {
        self.current_log_id += 1;
        let entry = LogEntry::new(self.current_log_id, operation);

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)?;

        let json = serde_json::to_string(&entry)?;
        use std::io::Write;
        writeln!(file, "{}", json)?;

        Ok(())
    }

    /// 创建快照
    pub fn create_snapshot(&self, tables: Vec<Table>) -> Result<()> {
        let snapshot = Snapshot::new(tables, self.current_log_id);
        let json = serde_json::to_string_pretty(&snapshot)?;
        fs::write(&self.snapshot_file, json)?;
        Ok(())
    }

    /// 加载快照
    pub fn load_snapshot(&self) -> Result<Option<Snapshot>> {
        if !Path::new(&self.snapshot_file).exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.snapshot_file)?;
        let snapshot: Snapshot = serde_json::from_str(&content)?;
        Ok(Some(snapshot))
    }

    /// 重放日志
    pub fn replay_logs(&self, from_id: u64) -> Result<Vec<LogEntry>> {
        if !Path::new(&self.log_file).exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.log_file)?;
        let mut entries = Vec::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            if let Ok(entry) = serde_json::from_str::<LogEntry>(line) {
                if entry.id > from_id {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    /// 清理旧日志
    pub fn cleanup_logs(&self) -> Result<()> {
        if Path::new(&self.log_file).exists() {
            fs::remove_file(&self.log_file)?;
        }
        Ok(())
    }

    /// 获取数据目录中的所有表
    pub fn list_tables(&self) -> Result<Vec<String>> {
        if !Path::new(&self.data_dir).exists() {
            return Ok(Vec::new());
        }

        let mut tables = Vec::new();
        for entry in fs::read_dir(&self.data_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                    if file_name != "snapshot" {
                        tables.push(file_name.to_string());
                    }
                }
            }
        }

        Ok(tables)
    }

    /// 备份数据库
    pub fn backup(&self, backup_path: &str) -> Result<()> {
        if !Path::new(backup_path).exists() {
            fs::create_dir_all(backup_path)?;
        }

        let backup_log_file = format!("{}/transaction.log", backup_path);
        let backup_snapshot_file = format!("{}/snapshot.json", backup_path);

        if Path::new(&self.log_file).exists() {
            fs::copy(&self.log_file, &backup_log_file)?;
        }

        if Path::new(&self.snapshot_file).exists() {
            fs::copy(&self.snapshot_file, &backup_snapshot_file)?;
        }

        Ok(())
    }

    /// 恢复数据库
    pub fn restore(&self, backup_path: &str) -> Result<()> {
        let backup_log_file = format!("{}/transaction.log", backup_path);
        let backup_snapshot_file = format!("{}/snapshot.json", backup_path);

        if Path::new(&backup_log_file).exists() {
            fs::copy(&backup_log_file, &self.log_file)?;
        }

        if Path::new(&backup_snapshot_file).exists() {
            fs::copy(&backup_snapshot_file, &self.snapshot_file)?;
        }

        Ok(())
    }

    /// 获取存储统计信息
    pub fn get_stats(&self) -> Result<StorageStats> {
        let mut stats = StorageStats::new();

        if Path::new(&self.log_file).exists() {
            let metadata = fs::metadata(&self.log_file)?;
            stats.log_file_size = metadata.len();
            let content = fs::read_to_string(&self.log_file)?;
            stats.total_log_entries = content.lines().count();
        }

        if Path::new(&self.snapshot_file).exists() {
            let metadata = fs::metadata(&self.snapshot_file)?;
            stats.snapshot_file_size = metadata.len();
        }

        Ok(stats)
    }
}

/// 存储统计信息
#[derive(Debug, Default)]
pub struct StorageStats {
    pub log_file_size: u64,
    pub snapshot_file_size: u64,
    pub total_log_entries: usize,
    pub current_log_id: u64,
}

impl StorageStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn total_size(&self) -> u64 {
        self.log_file_size + self.snapshot_file_size
    }
}

/// 内存存储后端
pub struct MemoryStorage {
    tables: std::collections::HashMap<String, Table>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            tables: std::collections::HashMap::new(),
        }
    }

    pub fn create_table(&mut self, name: &str, schema: Schema) -> Result<()> {
        if self.tables.contains_key(name) {
            return Err(DatabaseError::TableExists(name.to_string()));
        }

        self.tables.insert(name.to_string(), Table::new(name.to_string(), schema));
        Ok(())
    }

    pub fn get_table(&self, name: &str) -> Option<&Table> {
        self.tables.get(name)
    }

    pub fn get_table_mut(&mut self, name: &str) -> Option<&mut Table> {
        self.tables.get_mut(name)
    }

    pub fn drop_table(&mut self, name: &str) -> Result<()> {
        if self.tables.remove(name).is_none() {
            return Err(DatabaseError::TableNotFound(name.to_string()));
        }
        Ok(())
    }

    pub fn list_tables(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }

    pub fn insert_row(&mut self, table_name: &str, row: Row) -> Result<()> {
        if let Some(table) = self.tables.get_mut(table_name) {
            table.insert(row)?;
            Ok(())
        } else {
            Err(DatabaseError::TableNotFound(table_name.to_string()))
        }
    }

    pub fn update_row(&mut self, table_name: &str, id: uuid::Uuid, updates: std::collections::HashMap<String, Value>) -> Result<()> {
        if let Some(table) = self.tables.get_mut(table_name) {
            table.update(id, updates)?;
            Ok(())
        } else {
            Err(DatabaseError::TableNotFound(table_name.to_string()))
        }
    }

    pub fn delete_row(&mut self, table_name: &str, id: uuid::Uuid) -> Result<()> {
        if let Some(table) = self.tables.get_mut(table_name) {
            table.delete(id)?;
            Ok(())
        } else {
            Err(DatabaseError::TableNotFound(table_name.to_string()))
        }
    }

    pub fn get_all_data(&self) -> Vec<Table> {
        self.tables.values().cloned().collect()
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ColumnDefinition;

    #[test]
    fn test_memory_storage() {
        let mut storage = MemoryStorage::new();

        let schema = Schema::new(vec![
            ColumnDefinition::new("id", crate::types::DataType::Integer, true),
        ]);

        assert!(storage.create_table("test", schema.clone()).is_ok());
        assert!(storage.create_table("test", schema).is_err());
        assert_eq!(storage.list_tables(), vec!["test"]);
    }

    #[test]
    fn test_storage_engine() {
        let engine = StorageEngine::new();
        assert!(engine.initialize().is_ok());

        let stats = engine.get_stats();
        assert!(stats.is_ok());
    }

    #[test]
    fn test_log_entry_serialization() {
        let operation = StorageOperation::Create {
            table: "test".to_string(),
            schema: Schema::new(vec![]),
        };

        let entry = LogEntry::new(1, operation);
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: LogEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry.id, deserialized.id);
    }
}