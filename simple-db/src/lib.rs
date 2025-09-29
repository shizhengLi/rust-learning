pub mod error;
pub mod storage;
pub mod query;
pub mod types;
pub mod engine;

pub use error::{DatabaseError, Result};
pub use storage::StorageEngine;
pub use query::{Query, QueryResult, QueryEngine};
pub use types::{Value, Row, Table, Schema, DataType};
pub use engine::DatabaseEngine;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 主数据库结构
pub struct Database {
    tables: Arc<RwLock<HashMap<String, Table>>>,
    _storage: Arc<StorageEngine>,
}

impl Database {
    /// 创建新的数据库实例
    pub fn new() -> Self {
        Self {
            tables: Arc::new(RwLock::new(HashMap::new())),
            _storage: Arc::new(StorageEngine::new()),
        }
    }

    /// 创建表
    pub async fn create_table(&self, name: String, schema: Schema) -> Result<()> {
        let mut tables = self.tables.write().await;

        if tables.contains_key(&name) {
            return Err(DatabaseError::TableExists(name));
        }

        let table = Table::new(name.clone(), schema);
        tables.insert(name, table);

        Ok(())
    }

    /// 删除表
    pub async fn drop_table(&self, name: &str) -> Result<()> {
        let mut tables = self.tables.write().await;

        if tables.remove(name).is_none() {
            return Err(DatabaseError::TableNotFound(name.to_string()));
        }

        Ok(())
    }

    /// 获取表
    pub async fn get_table(&self, name: &str) -> Result<Table> {
        let tables = self.tables.read().await;
        tables.get(name)
            .cloned()
            .ok_or_else(|| DatabaseError::TableNotFound(name.to_string()))
    }

    /// 插入数据
    pub async fn insert(&self, table_name: &str, row: Row) -> Result<()> {
        let mut tables = self.tables.write().await;

        let table = tables.get_mut(table_name)
            .ok_or_else(|| DatabaseError::TableNotFound(table_name.to_string()))?;

        table.insert(row)?;

        Ok(())
    }

    /// 查询数据
    pub async fn query(&self, query: Query) -> Result<QueryResult> {
        let tables = self.tables.read().await;

        let table = tables.get(&query.table_name)
            .ok_or_else(|| DatabaseError::TableNotFound(query.table_name.clone()))?;

        let engine = QueryEngine::new();
        engine.execute(table.clone(), query).await
    }

    /// 列出所有表
    pub async fn list_tables(&self) -> Vec<String> {
        let tables = self.tables.read().await;
        tables.keys().cloned().collect()
    }

    /// 获取表结构
    pub async fn get_schema(&self, table_name: &str) -> Result<Schema> {
        let table = self.get_table(table_name).await?;
        Ok(table.schema().clone())
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ColumnDefinition, DataType};

    #[tokio::test]
    async fn test_create_table() {
        let db = Database::new();

        let schema = Schema::new(vec![
            ColumnDefinition::new("id", DataType::Integer, true),
            ColumnDefinition::new("name", DataType::Text, false),
        ]);

        db.create_table("users".to_string(), schema).await.unwrap();

        let tables = db.list_tables().await;
        assert_eq!(tables, vec!["users"]);
    }

    #[tokio::test]
    async fn test_insert_and_query() {
        let db = Database::new();

        let schema = Schema::new(vec![
            ColumnDefinition::new("id", DataType::Integer, true),
            ColumnDefinition::new("name", DataType::Text, false),
        ]);

        db.create_table("users".to_string(), schema).await.unwrap();

        let mut row = Row::new();
        row.set("id", Value::Integer(1));
        row.set("name", Value::Text("Alice".to_string()));

        db.insert("users", row).await.unwrap();

        let query = Query::select("users".to_string());
        let result = db.query(query).await.unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].get("id"), Some(&Value::Integer(1)));
        assert_eq!(result.rows[0].get("name"), Some(&Value::Text("Alice".to_string())));
    }
}