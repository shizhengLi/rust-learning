use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{DatabaseError, Result};

/// 数据类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    Integer,
    Text,
    Boolean,
    Float,
    Date,
    Time,
    DateTime,
    Json,
    Binary,
}

impl DataType {
    pub fn to_string(&self) -> String {
        match self {
            DataType::Integer => "INTEGER".to_string(),
            DataType::Text => "TEXT".to_string(),
            DataType::Boolean => "BOOLEAN".to_string(),
            DataType::Float => "FLOAT".to_string(),
            DataType::Date => "DATE".to_string(),
            DataType::Time => "TIME".to_string(),
            DataType::DateTime => "DATETIME".to_string(),
            DataType::Json => "JSON".to_string(),
            DataType::Binary => "BINARY".to_string(),
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "INTEGER" | "INT" => Ok(DataType::Integer),
            "TEXT" | "STRING" | "VARCHAR" => Ok(DataType::Text),
            "BOOLEAN" | "BOOL" => Ok(DataType::Boolean),
            "FLOAT" | "DOUBLE" | "REAL" => Ok(DataType::Float),
            "DATE" => Ok(DataType::Date),
            "TIME" => Ok(DataType::Time),
            "DATETIME" | "TIMESTAMP" => Ok(DataType::DateTime),
            "JSON" => Ok(DataType::Json),
            "BINARY" | "BLOB" => Ok(DataType::Binary),
            _ => Err(DatabaseError::parse_error(format!("未知数据类型: {}", s))),
        }
    }
}

/// 数据值
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Integer(i64),
    Text(String),
    Boolean(bool),
    Float(f64),
    Date(chrono::NaiveDate),
    Time(chrono::NaiveTime),
    DateTime(chrono::NaiveDateTime),
    Json(serde_json::Value),
    Binary(Vec<u8>),
    Null,
}

impl Value {
    pub fn get_type(&self) -> DataType {
        match self {
            Value::Integer(_) => DataType::Integer,
            Value::Text(_) => DataType::Text,
            Value::Boolean(_) => DataType::Boolean,
            Value::Float(_) => DataType::Float,
            Value::Date(_) => DataType::Date,
            Value::Time(_) => DataType::Time,
            Value::DateTime(_) => DataType::DateTime,
            Value::Json(_) => DataType::Json,
            Value::Binary(_) => DataType::Binary,
            Value::Null => DataType::Text, // NULL 可以是任何类型，默认为 Text
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::Integer(i) => i.to_string(),
            Value::Text(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Date(d) => d.to_string(),
            Value::Time(t) => t.to_string(),
            Value::DateTime(dt) => dt.to_string(),
            Value::Json(j) => j.to_string(),
            Value::Binary(b) => format!("BINARY({} bytes)", b.len()),
            Value::Null => "NULL".to_string(),
        }
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Integer(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::Text(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::Text(value.to_string())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl From<serde_json::Value> for Value {
    fn from(value: serde_json::Value) -> Self {
        Value::Json(value)
    }
}

/// 列定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub unique: bool,
    pub default_value: Option<Value>,
    pub primary_key: bool,
}

impl ColumnDefinition {
    pub fn new<S: Into<String>>(name: S, data_type: DataType, primary_key: bool) -> Self {
        Self {
            name: name.into(),
            data_type,
            nullable: !primary_key, // 主键默认不允许为空
            unique: primary_key,   // 主键默认唯一
            default_value: None,
            primary_key,
        }
    }

    pub fn nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }

    pub fn unique(mut self, unique: bool) -> Self {
        self.unique = unique;
        self
    }

    pub fn default_value(mut self, value: Value) -> Self {
        self.default_value = Some(value);
        self
    }
}

/// 表结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub columns: Vec<ColumnDefinition>,
}

impl Schema {
    pub fn new(columns: Vec<ColumnDefinition>) -> Self {
        Self { columns }
    }

    pub fn get_column(&self, name: &str) -> Option<&ColumnDefinition> {
        self.columns.iter().find(|col| col.name == name)
    }

    pub fn get_primary_key_columns(&self) -> Vec<&ColumnDefinition> {
        self.columns.iter().filter(|col| col.primary_key).collect()
    }

    pub fn validate_row(&self, row: &Row) -> Result<()> {
        // 检查必填字段
        for column in &self.columns {
            if !column.nullable && !column.primary_key {
                if row.get(&column.name).map_or(true, |v| v.is_null()) {
                    if column.default_value.is_none() {
                        return Err(DatabaseError::not_null_violation(
                            format!("列 '{}' 不能为空", column.name)
                        ));
                    }
                }
            }
        }

        // 检查主键
        let pk_columns = self.get_primary_key_columns();
        if !pk_columns.is_empty() {
            let mut has_pk = false;
            for pk_col in pk_columns {
                if let Some(value) = row.get(&pk_col.name) {
                    if !value.is_null() {
                        has_pk = true;
                        break;
                    }
                }
            }
            if !has_pk {
                return Err(DatabaseError::not_null_violation(
                    "主键不能为空".to_string()
                ));
            }
        }

        Ok(())
    }
}

/// 数据行
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    pub id: Uuid,
    pub data: HashMap<String, Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Row {
    pub fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            data: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn set<S: Into<String>>(&mut self, column: S, value: Value) {
        self.data.insert(column.into(), value);
    }

    pub fn get(&self, column: &str) -> Option<&Value> {
        self.data.get(column)
    }

    pub fn get_integer(&self, column: &str) -> Option<i64> {
        match self.get(column) {
            Some(Value::Integer(i)) => Some(*i),
            _ => None,
        }
    }

    pub fn get_text(&self, column: &str) -> Option<&str> {
        match self.get(column) {
            Some(Value::Text(s)) => Some(s),
            _ => None,
        }
    }

    pub fn get_boolean(&self, column: &str) -> Option<bool> {
        match self.get(column) {
            Some(Value::Boolean(b)) => Some(*b),
            _ => None,
        }
    }

    pub fn get_float(&self, column: &str) -> Option<f64> {
        match self.get(column) {
            Some(Value::Float(f)) => Some(*f),
            _ => None,
        }
    }

    pub fn columns(&self) -> Vec<&str> {
        self.data.keys().map(|s| s.as_str()).collect()
    }
}

/// 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    pub schema: Schema,
    pub rows: Vec<Row>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Table {
    pub fn new(name: String, schema: Schema) -> Self {
        Self {
            name,
            schema,
            rows: Vec::new(),
            created_at: chrono::Utc::now(),
        }
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn insert(&mut self, mut row: Row) -> Result<()> {
        // 验证行数据
        self.schema.validate_row(&row)?;

        // 设置默认值
        for column in &self.schema.columns {
            if !row.data.contains_key(&column.name) {
                if let Some(default_value) = &column.default_value {
                    row.set(column.name.clone(), default_value.clone());
                }
            }
        }

        // 检查唯一约束
        if column_has_unique_constraint(&self.schema) {
            for existing_row in &self.rows {
                for column in &self.schema.columns {
                    if column.unique {
                        if let (Some(new_val), Some(existing_val)) =
                            (row.get(&column.name), existing_row.get(&column.name)) {
                            if new_val == existing_val && !new_val.is_null() {
                                return Err(DatabaseError::unique_violation(
                                    format!("列 '{}' 的值 '{}' 必须唯一", column.name, new_val.to_string())
                                ));
                            }
                        }
                    }
                }
            }
        }

        self.rows.push(row);
        Ok(())
    }

    pub fn find_by_id(&self, id: Uuid) -> Option<&Row> {
        self.rows.iter().find(|row| row.id == id)
    }

    pub fn update(&mut self, id: Uuid, updates: HashMap<String, Value>) -> Result<()> {
        if let Some(row) = self.rows.iter_mut().find(|row| row.id == id) {
            for (column, value) in updates {
                row.set(column, value);
            }
            row.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err(DatabaseError::Other(format!("未找到ID为 {} 的行", id)))
        }
    }

    pub fn delete(&mut self, id: Uuid) -> Result<()> {
        let initial_len = self.rows.len();
        self.rows.retain(|row| row.id != id);

        if self.rows.len() == initial_len {
            Err(DatabaseError::Other(format!("未找到ID为 {} 的行", id)))
        } else {
            Ok(())
        }
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }
}

fn column_has_unique_constraint(schema: &Schema) -> bool {
    schema.columns.iter().any(|col| col.unique || col.primary_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_conversions() {
        let int_val: Value = 42.into();
        assert_eq!(int_val, Value::Integer(42));

        let text_val: Value = "hello".into();
        assert_eq!(text_val, Value::Text("hello".to_string()));

        let bool_val: Value = true.into();
        assert_eq!(bool_val, Value::Boolean(true));
    }

    #[test]
    fn test_schema_validation() {
        let schema = Schema::new(vec![
            ColumnDefinition::new("id", DataType::Integer, true),
            ColumnDefinition::new("name", DataType::Text, false).nullable(false),
        ]);

        let mut row = Row::new();
        row.set("id", Value::Integer(1));

        // 应该失败，因为 name 是必填字段
        assert!(schema.validate_row(&row).is_err());

        row.set("name", Value::Text("Alice".to_string()));
        assert!(schema.validate_row(&row).is_ok());
    }

    #[test]
    fn test_table_operations() {
        let schema = Schema::new(vec![
            ColumnDefinition::new("id", DataType::Integer, true),
            ColumnDefinition::new("name", DataType::Text, false),
        ]);

        let mut table = Table::new("users".to_string(), schema);

        let mut row = Row::new();
        row.set("id", Value::Integer(1));
        row.set("name", Value::Text("Alice".to_string()));

        assert!(table.insert(row).is_ok());
        assert_eq!(table.row_count(), 1);
    }
}