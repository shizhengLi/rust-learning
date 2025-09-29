use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

use crate::error::{DatabaseError, Result};
use crate::types::{Row, Schema, Value};
use crate::query::{Query, QueryResult, QueryEngine, QueryBuilder, ComparisonOperator};
use crate::storage::{StorageEngine, MemoryStorage, StorageOperation};

/// 数据库引擎 - 提供高级数据库操作接口
pub struct DatabaseEngine {
    storage: Arc<RwLock<MemoryStorage>>,
    disk_storage: Arc<Mutex<StorageEngine>>,
    auto_save: bool,
}

impl DatabaseEngine {
    /// 创建新的数据库引擎
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(MemoryStorage::new())),
            disk_storage: Arc::new(Mutex::new(StorageEngine::new())),
            auto_save: true,
        }
    }

    /// 从磁盘加载数据库
    pub async fn load_from_disk() -> Result<Self> {
        let engine = Self::new();
        engine.disk_storage.lock().unwrap().initialize()?;

        // 加载快照
        let snapshot = engine.disk_storage.lock().unwrap().load_snapshot()?;
        if let Some(ref snapshot_data) = snapshot {
            let mut storage = engine.storage.write().await;
            for table in &snapshot_data.tables {
                storage.create_table(&table.name, table.schema.clone())?;
            }
        }

        // 重放日志
        let last_log_id = snapshot.as_ref().map(|s| s.last_log_id).unwrap_or(0);
        let logs = engine.disk_storage.lock().unwrap().replay_logs(last_log_id)?;
        {
            let mut storage = engine.storage.write().await;
            for log in logs {
                engine.apply_log_operation(&mut storage, log.operation)?;
            }
        } // storage borrow ends here

        Ok(engine)
    }

    /// 保存到磁盘
    pub async fn save_to_disk(&self) -> Result<()> {
        let storage = self.storage.read().await;
        let tables = storage.get_all_data();
        self.disk_storage.lock().unwrap().create_snapshot(tables)?;
        Ok(())
    }

    /// 设置自动保存
    pub fn set_auto_save(&mut self, auto_save: bool) {
        self.auto_save = auto_save;
    }

    /// 创建表
    pub async fn create_table(&self, name: &str, schema: Schema) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.create_table(name, schema.clone())?;

        // 记录操作日志
        if self.auto_save {
            self.disk_storage.lock().unwrap().write_log(StorageOperation::Create {
                table: name.to_string(),
                schema,
            })?;
        }

        Ok(())
    }

    /// 删除表
    pub async fn drop_table(&self, name: &str) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.drop_table(name)?;

        // 记录操作日志
        if self.auto_save {
            self.disk_storage.lock().unwrap().write_log(StorageOperation::Drop {
                table: name.to_string(),
            })?;
        }

        Ok(())
    }

    /// 插入数据
    pub async fn insert(&self, table_name: &str, data: HashMap<String, Value>) -> Result<uuid::Uuid> {
        let mut row = Row::new();
        for (column, value) in data {
            row.set(column, value);
        }

        let row_id = row.id;
        let mut storage = self.storage.write().await;
        storage.insert_row(table_name, row.clone())?;

        // 记录操作日志
        if self.auto_save {
            let _operation_data: std::collections::HashMap<String, Value> = row.data.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            self.disk_storage.lock().unwrap().write_log(StorageOperation::Insert {
                table: table_name.to_string(),
                row,
            })?;
        }

        Ok(row_id)
    }

    /// 查询数据
    pub async fn query(&self, query: Query) -> Result<QueryResult> {
        let storage = self.storage.read().await;
        let table = storage.get_table(&query.table_name)
            .ok_or_else(|| DatabaseError::TableNotFound(query.table_name.clone()))?;

        let engine = QueryEngine::new();
        engine.execute(table.clone(), query).await
    }

    /// 更新数据
    pub async fn update(&self, table_name: &str, conditions: Vec<(String, ComparisonOperator, Value)>, updates: HashMap<String, Value>) -> Result<usize> {
        let _query = QueryBuilder::update(table_name, updates.clone()).build();

        let mut storage = self.storage.write().await;
        let table = storage.get_table_mut(table_name)
            .ok_or_else(|| DatabaseError::TableNotFound(table_name.to_string()))?;

        let mut affected_count = 0;

        for row in &mut table.rows {
            let matches = conditions.iter().all(|(column, operator, value)| {
                let condition = crate::query::Condition::new(column, operator.clone(), value.clone());
                condition.evaluate(row).unwrap_or(false)
            });

            if matches {
                let row_updates: std::collections::HashMap<String, Value> = updates.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                // Update the row directly since we have mutable access
                for (key, value) in row_updates {
                    row.set(&key, value);
                }
                row.updated_at = chrono::Utc::now();
                affected_count += 1;
            }
        }

        // 记录操作日志
        if self.auto_save && affected_count > 0 {
            for row in &table.rows {
                let matches = conditions.iter().all(|(column, operator, value)| {
                    let condition = crate::query::Condition::new(column, operator.clone(), value.clone());
                    condition.evaluate(row).unwrap_or(false)
                });

                if matches {
                    let operation_data = updates.iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();
                    self.disk_storage.lock().unwrap().write_log(StorageOperation::Update {
                        table: table_name.to_string(),
                        id: row.id.to_string(),
                        data: operation_data,
                    })?;
                }
            }
        }

        Ok(affected_count)
    }

    /// 删除数据
    pub async fn delete(&self, table_name: &str, conditions: Vec<(String, ComparisonOperator, Value)>) -> Result<usize> {
        let mut storage = self.storage.write().await;
        let table = storage.get_table_mut(table_name)
            .ok_or_else(|| DatabaseError::TableNotFound(table_name.to_string()))?;

        let mut affected_count = 0;
        let mut rows_to_delete = Vec::new();

        for row in &table.rows {
            let matches = conditions.iter().all(|(column, operator, value)| {
                let condition = crate::query::Condition::new(column, operator.clone(), value.clone());
                condition.evaluate(row).unwrap_or(false)
            });

            if matches {
                rows_to_delete.push(row.id);
                affected_count += 1;
            }
        }

        for row_id in rows_to_delete {
            storage.delete_row(table_name, row_id)?;

            // 记录操作日志
            if self.auto_save {
                self.disk_storage.lock().unwrap().write_log(StorageOperation::Delete {
                    table: table_name.to_string(),
                    id: row_id.to_string(),
                })?;
            }
        }

        Ok(affected_count)
    }

    /// 获取表信息
    pub async fn get_table_info(&self, table_name: &str) -> Result<TableInfo> {
        let storage = self.storage.read().await;
        let table = storage.get_table(table_name)
            .ok_or_else(|| DatabaseError::TableNotFound(table_name.to_string()))?;

        Ok(TableInfo {
            name: table.name.clone(),
            row_count: table.row_count(),
            created_at: table.created_at,
            schema: table.schema().clone(),
        })
    }

    /// 列出所有表
    pub async fn list_tables(&self) -> Vec<TableInfo> {
        let storage = self.storage.read().await;
        let mut tables = Vec::new();

        for table_name in storage.list_tables() {
            if let Some(table) = storage.get_table(&table_name) {
                tables.push(TableInfo {
                    name: table.name.clone(),
                    row_count: table.row_count(),
                    created_at: table.created_at,
                    schema: table.schema().clone(),
                });
            }
        }

        tables
    }

    /// 获取数据库统计信息
    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        let storage = self.storage.read().await;
        let storage_stats = self.disk_storage.lock().unwrap().get_stats()?;

        Ok(DatabaseStats {
            total_tables: storage.list_tables().len(),
            total_rows: storage.list_tables().iter()
                .filter_map(|name| storage.get_table(name))
                .map(|table| table.row_count())
                .sum(),
            storage_stats,
        })
    }

    /// 备份数据库
    pub async fn backup(&self, backup_path: &str) -> Result<()> {
        // 先保存当前状态
        self.save_to_disk().await?;
        self.disk_storage.lock().unwrap().backup(backup_path)?;
        Ok(())
    }

    /// 恢复数据库
    pub async fn restore(&self, backup_path: &str) -> Result<()> {
        self.disk_storage.lock().unwrap().restore(backup_path)?;

        // 重新加载数据
        let logs = self.disk_storage.lock().unwrap().replay_logs(0)?;
        let mut storage = self.storage.write().await;

        // 清空当前数据
        let table_names: Vec<String> = storage.list_tables();
        for table_name in table_names {
            storage.drop_table(&table_name)?;
        }

        // 重放日志
        for log in logs {
            self.apply_log_operation(&mut storage, log.operation)?;
        }

        Ok(())
    }

    /// 应用日志操作
    fn apply_log_operation(&self, storage: &mut MemoryStorage, operation: StorageOperation) -> Result<()> {
        match operation {
            StorageOperation::Create { table, schema } => {
                storage.create_table(&table, schema)?;
            }
            StorageOperation::Insert { table, row } => {
                storage.insert_row(&table, row)?;
            }
            StorageOperation::Update { table, id, data } => {
                if let Ok(uuid) = uuid::Uuid::parse_str(&id) {
                    let updates = data.into_iter().collect();
                    storage.update_row(&table, uuid, updates)?;
                }
            }
            StorageOperation::Delete { table, id } => {
                if let Ok(uuid) = uuid::Uuid::parse_str(&id) {
                    storage.delete_row(&table, uuid)?;
                }
            }
            StorageOperation::Drop { table } => {
                storage.drop_table(&table)?;
            }
        }
        Ok(())
    }

    /// 执行事务
    pub async fn transaction<F, T>(&self, operations: F) -> Result<T>
    where
        F: FnOnce(&mut Transaction) -> Result<T>,
    {
        let mut transaction = Transaction::new(self);
        let result = operations(&mut transaction)?;

        // 提交事务
        transaction.commit().await?;

        Ok(result)
    }

    /// 批量插入
    pub async fn batch_insert(&self, table_name: &str, rows: Vec<HashMap<String, Value>>) -> Result<Vec<uuid::Uuid>> {
        let mut ids = Vec::new();

        for row_data in rows {
            let id = self.insert(table_name, row_data).await?;
            ids.push(id);
        }

        Ok(ids)
    }

    /// 清空表
    pub async fn truncate_table(&self, table_name: &str) -> Result<usize> {
        let storage = self.storage.read().await;
        let table = storage.get_table(table_name)
            .ok_or_else(|| DatabaseError::TableNotFound(table_name.to_string()))?;

        let _count = table.row_count();

        // 删除所有行
        let query = QueryBuilder::delete(table_name).build();
        let result = self.query(query).await?;

        Ok(result.affected_rows)
    }
}

impl Default for DatabaseEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// 表信息
#[derive(Debug, Clone)]
pub struct TableInfo {
    pub name: String,
    pub row_count: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub schema: Schema,
}

/// 数据库统计信息
#[derive(Debug)]
pub struct DatabaseStats {
    pub total_tables: usize,
    pub total_rows: usize,
    pub storage_stats: crate::storage::StorageStats,
}

/// 事务对象
pub struct Transaction<'a> {
    engine: &'a DatabaseEngine,
    operations: Vec<StorageOperation>,
}

impl<'a> Transaction<'a> {
    fn new(engine: &'a DatabaseEngine) -> Self {
        Self {
            engine,
            operations: Vec::new(),
        }
    }

    /// 在事务中创建表
    pub fn create_table(&mut self, name: &str, schema: Schema) -> Result<()> {
        self.operations.push(StorageOperation::Create {
            table: name.to_string(),
            schema,
        });
        Ok(())
    }

    /// 在事务中插入数据
    pub fn insert(&mut self, table_name: &str, data: HashMap<String, Value>) -> Result<uuid::Uuid> {
        let mut row = Row::new();
        for (column, value) in data {
            row.set(column, value);
        }

        self.operations.push(StorageOperation::Insert {
            table: table_name.to_string(),
            row: row.clone(),
        });

        Ok(row.id)
    }

    /// 在事务中更新数据
    pub fn update(&mut self, table_name: &str, id: uuid::Uuid, updates: HashMap<String, Value>) -> Result<()> {
        let operation_data = updates.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        self.operations.push(StorageOperation::Update {
            table: table_name.to_string(),
            id: id.to_string(),
            data: operation_data,
        });
        Ok(())
    }

    /// 在事务中删除数据
    pub fn delete(&mut self, table_name: &str, id: uuid::Uuid) -> Result<()> {
        self.operations.push(StorageOperation::Delete {
            table: table_name.to_string(),
            id: id.to_string(),
        });
        Ok(())
    }

    /// 提交事务
    pub async fn commit(self) -> Result<()> {
        let mut storage = self.engine.storage.write().await;

        // 执行所有操作
        for operation in self.operations {
            self.engine.apply_log_operation(&mut storage, operation.clone())?;

            // 记录到磁盘
            if self.engine.auto_save {
                self.engine.disk_storage.lock().unwrap().write_log(operation)?;
            }
        }

        // 如果启用了自动保存，创建快照
        if self.engine.auto_save {
            self.engine.save_to_disk().await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ColumnDefinition, DataType};

    #[tokio::test]
    async fn test_database_engine() {
        let engine = DatabaseEngine::new();

        // 创建表
        let schema = Schema::new(vec![
            ColumnDefinition::new("id", DataType::Integer, true),
            ColumnDefinition::new("name", DataType::Text, false),
        ]);

        engine.create_table("users", schema).await.unwrap();

        // 插入数据
        let mut data = HashMap::new();
        data.insert("id".to_string(), Value::Integer(1));
        data.insert("name".to_string(), Value::Text("Alice".to_string()));

        let _id = engine.insert("users", data).await.unwrap();

        // 查询数据
        let query = QueryBuilder::select("users")
            .where_condition("id", ComparisonOperator::Equal, Value::Integer(1))
            .build();

        let result = engine.query(query).await.unwrap();
        assert_eq!(result.rows.len(), 1);

        // 获取表信息
        let table_info = engine.get_table_info("users").await.unwrap();
        assert_eq!(table_info.row_count, 1);
    }

    #[tokio::test]
    async fn test_transaction() {
        let mut engine = DatabaseEngine::new();
        engine.set_auto_save(false);

        let schema = Schema::new(vec![
            ColumnDefinition::new("id", DataType::Integer, true),
            ColumnDefinition::new("value", DataType::Text, false),
        ]);

        let result = engine.transaction(|tx| {
            tx.create_table("test_table", schema.clone())?;

            let mut data1 = HashMap::new();
            data1.insert("id".to_string(), Value::Integer(1));
            data1.insert("value".to_string(), Value::Text("test1".to_string()));
            tx.insert("test_table", data1)?;

            let mut data2 = HashMap::new();
            data2.insert("id".to_string(), Value::Integer(2));
            data2.insert("value".to_string(), Value::Text("test2".to_string()));
            tx.insert("test_table", data2)?;

            Ok::<(), DatabaseError>(())
        }).await;

        assert!(result.is_ok());

        // 验证数据
        let tables = engine.list_tables().await;
        assert!(tables.iter().any(|t| t.name == "test_table"));
    }

    #[tokio::test]
    async fn test_batch_operations() {
        let engine = DatabaseEngine::new();

        let schema = Schema::new(vec![
            ColumnDefinition::new("id", DataType::Integer, true),
            ColumnDefinition::new("name", DataType::Text, false),
        ]);

        engine.create_table("batch_test", schema).await.unwrap();

        // 批量插入
        let mut rows = Vec::new();
        for i in 1..=5 {
            let mut data = HashMap::new();
            data.insert("id".to_string(), Value::Integer(i));
            data.insert("name".to_string(), Value::Text(format!("User {}", i)));
            rows.push(data);
        }

        let ids = engine.batch_insert("batch_test", rows).await.unwrap();
        assert_eq!(ids.len(), 5);

        // 统计信息
        let stats = engine.get_stats().await.unwrap();
        assert_eq!(stats.total_rows, 5);
    }
}