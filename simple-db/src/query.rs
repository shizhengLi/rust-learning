use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::{DatabaseError, Result};
use crate::types::{Value, Table, Row};

/// 查询条件运算符
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Like,
    In,
    IsNull,
    IsNotNull,
}

impl ComparisonOperator {
    pub fn to_string(&self) -> String {
        match self {
            ComparisonOperator::Equal => "=".to_string(),
            ComparisonOperator::NotEqual => "!=".to_string(),
            ComparisonOperator::GreaterThan => ">".to_string(),
            ComparisonOperator::GreaterThanOrEqual => ">=".to_string(),
            ComparisonOperator::LessThan => "<".to_string(),
            ComparisonOperator::LessThanOrEqual => "<=".to_string(),
            ComparisonOperator::Like => "LIKE".to_string(),
            ComparisonOperator::In => "IN".to_string(),
            ComparisonOperator::IsNull => "IS NULL".to_string(),
            ComparisonOperator::IsNotNull => "IS NOT NULL".to_string(),
        }
    }
}

/// 查询条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub column: String,
    pub operator: ComparisonOperator,
    pub value: Value,
}

impl Condition {
    pub fn new<S: Into<String>>(column: S, operator: ComparisonOperator, value: Value) -> Self {
        Self {
            column: column.into(),
            operator,
            value,
        }
    }

    pub fn evaluate(&self, row: &Row) -> Result<bool> {
        let row_value = row.get(&self.column);

        match self.operator {
            ComparisonOperator::Equal => Ok(self.compare_values(row_value, &self.value)? == 0),
            ComparisonOperator::NotEqual => Ok(self.compare_values(row_value, &self.value)? != 0),
            ComparisonOperator::GreaterThan => Ok(self.compare_values(row_value, &self.value)? > 0),
            ComparisonOperator::GreaterThanOrEqual => Ok(self.compare_values(row_value, &self.value)? >= 0),
            ComparisonOperator::LessThan => Ok(self.compare_values(row_value, &self.value)? < 0),
            ComparisonOperator::LessThanOrEqual => Ok(self.compare_values(row_value, &self.value)? <= 0),
            ComparisonOperator::Like => Ok(self.evaluate_like(row_value)),
            ComparisonOperator::In => Ok(self.evaluate_in(row_value)),
            ComparisonOperator::IsNull => Ok(row_value.map_or(true, |v| v.is_null())),
            ComparisonOperator::IsNotNull => Ok(row_value.map_or(false, |v| !v.is_null())),
        }
    }

    fn compare_values(&self, a: Option<&Value>, b: &Value) -> Result<i32> {
        match (a, b) {
            (Some(Value::Integer(a)), Value::Integer(b)) => Ok(a.cmp(b) as i32),
            (Some(Value::Text(a)), Value::Text(b)) => Ok(a.cmp(b) as i32),
            (Some(Value::Boolean(a)), Value::Boolean(b)) => Ok(a.cmp(b) as i32),
            (Some(Value::Float(a)), Value::Float(b)) => {
                if a.partial_cmp(b).is_some() {
                    Ok(a.partial_cmp(b).unwrap() as i32)
                } else {
                    Ok(0)
                }
            }
            (Some(Value::Date(a)), Value::Date(b)) => Ok(a.cmp(b) as i32),
            (Some(Value::Time(a)), Value::Time(b)) => Ok(a.cmp(b) as i32),
            (Some(Value::DateTime(a)), Value::DateTime(b)) => Ok(a.cmp(b) as i32),
            (None, _) => Ok(-1), // NULL 值最小
            (Some(_), _) => Err(DatabaseError::type_mismatch(
                format!("无法比较列 '{}' 的值", self.column)
            )),
        }
    }

    fn evaluate_like(&self, row_value: Option<&Value>) -> bool {
        match (row_value, &self.value) {
            (Some(Value::Text(row_text)), Value::Text(pattern_text)) => {
                let pattern = pattern_text.replace("%", ".*").replace("_", ".");
                let regex = match regex::Regex::new(&format!("^{}$", pattern)) {
                    Ok(re) => re,
                    Err(_) => return false,
                };
                regex.is_match(row_text)
            }
            _ => false,
        }
    }

    fn evaluate_in(&self, row_value: Option<&Value>) -> bool {
        if let Value::Json(json_value) = &self.value {
            if let Some(array) = json_value.as_array() {
                if let Some(row_val) = row_value {
                    return array.iter().any(|item| {
                        let item_value = Value::from(item.clone());
                        self.compare_values(Some(row_val), &item_value).unwrap_or(0) == 0
                    });
                }
            }
        }
        false
    }
}

/// 排序规范
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBy {
    pub column: String,
    pub ascending: bool,
}

impl OrderBy {
    pub fn new<S: Into<String>>(column: S, ascending: bool) -> Self {
        Self {
            column: column.into(),
            ascending,
        }
    }
}

/// 查询类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryType {
    Select,
    Insert,
    Update,
    Delete,
    Count,
}

/// 查询对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub query_type: QueryType,
    pub table_name: String,
    pub conditions: Vec<Condition>,
    pub order_by: Vec<OrderBy>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub data: Option<HashMap<String, Value>>,
}

impl Query {
    pub fn select<S: Into<String>>(table_name: S) -> Self {
        Self {
            query_type: QueryType::Select,
            table_name: table_name.into(),
            conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            data: None,
        }
    }

    pub fn insert<S: Into<String>>(table_name: S, data: HashMap<String, Value>) -> Self {
        Self {
            query_type: QueryType::Insert,
            table_name: table_name.into(),
            conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            data: Some(data),
        }
    }

    pub fn update<S: Into<String>>(table_name: S, data: HashMap<String, Value>) -> Self {
        Self {
            query_type: QueryType::Update,
            table_name: table_name.into(),
            conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            data: Some(data),
        }
    }

    pub fn delete<S: Into<String>>(table_name: S) -> Self {
        Self {
            query_type: QueryType::Delete,
            table_name: table_name.into(),
            conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            data: None,
        }
    }

    pub fn count<S: Into<String>>(table_name: S) -> Self {
        Self {
            query_type: QueryType::Count,
            table_name: table_name.into(),
            conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            data: None,
        }
    }

    pub fn where_condition(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    pub fn order_by(mut self, order_by: OrderBy) -> Self {
        self.order_by.push(order_by);
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
}

/// 查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub query_type: QueryType,
    pub table_name: String,
    pub rows: Vec<Row>,
    pub affected_rows: usize,
    pub execution_time_ms: u64,
    pub count: Option<usize>,
}

impl QueryResult {
    pub fn new(query_type: QueryType, table_name: String, execution_time_ms: u64) -> Self {
        Self {
            query_type,
            table_name,
            rows: Vec::new(),
            affected_rows: 0,
            execution_time_ms,
            count: None,
        }
    }

    pub fn with_rows(mut self, rows: Vec<Row>) -> Self {
        let row_count = rows.len();
        self.rows = rows;
        self.affected_rows = row_count;
        self
    }

    pub fn with_count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self.affected_rows = count;
        self
    }

    pub fn with_affected_rows(mut self, affected_rows: usize) -> Self {
        self.affected_rows = affected_rows;
        self
    }
}

/// 查询引擎
pub struct QueryEngine;

impl QueryEngine {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, table: Table, query: Query) -> Result<QueryResult> {
        let start_time = std::time::Instant::now();

        let result = match query.query_type {
            QueryType::Select => self.execute_select(&table, &query).await,
            QueryType::Insert => self.execute_insert(&table, &query).await,
            QueryType::Update => self.execute_update(&table, &query).await,
            QueryType::Delete => self.execute_delete(&table, &query).await,
            QueryType::Count => self.execute_count(&table, &query).await,
        };

        let execution_time = start_time.elapsed().as_millis() as u64;
        result.map(|mut r| {
            r.execution_time_ms = execution_time;
            r
        })
    }

    async fn execute_select(&self, table: &Table, query: &Query) -> Result<QueryResult> {
        let mut filtered_rows = table.rows.clone();

        // 应用过滤条件
        if !query.conditions.is_empty() {
            filtered_rows.retain(|row| {
                query.conditions.iter().all(|condition| {
                    condition.evaluate(row).unwrap_or(false)
                })
            });
        }

        // 排序
        if !query.order_by.is_empty() {
            self.sort_rows(&mut filtered_rows, &query.order_by);
        }

        // 分页
        let start = query.offset.unwrap_or(0);
        let end = if let Some(limit) = query.limit {
            start + limit
        } else {
            filtered_rows.len()
        };

        let paginated_rows = if start < filtered_rows.len() {
            filtered_rows[start..end.min(filtered_rows.len())].to_vec()
        } else {
            Vec::new()
        };

        Ok(QueryResult::new(
            QueryType::Select,
            table.name.clone(),
            0,
        ).with_rows(paginated_rows))
    }

    async fn execute_insert(&self, table: &Table, query: &Query) -> Result<QueryResult> {
        if let Some(data) = &query.data {
            let mut row = Row::new();
            for (column, value) in data {
                row.set(column.clone(), value.clone());
            }

            // 在实际实现中，这里需要修改表数据
            // 由于表是不可变引用，我们返回操作信息
            Ok(QueryResult::new(
                QueryType::Insert,
                table.name.clone(),
                0,
            ).with_affected_rows(1))
        } else {
            Err(DatabaseError::Other("INSERT 查询缺少数据".to_string()))
        }
    }

    async fn execute_update(&self, table: &Table, query: &Query) -> Result<QueryResult> {
        let mut affected_count = 0;

        // 找到符合条件的行
        for row in &table.rows {
            let matches = query.conditions.iter().all(|condition| {
                condition.evaluate(row).unwrap_or(false)
            });

            if matches {
                affected_count += 1;
            }
        }

        Ok(QueryResult::new(
            QueryType::Update,
            table.name.clone(),
            0,
        ).with_affected_rows(affected_count))
    }

    async fn execute_delete(&self, table: &Table, query: &Query) -> Result<QueryResult> {
        let mut affected_count = 0;

        // 计算符合条件的行数
        for row in &table.rows {
            let matches = query.conditions.iter().all(|condition| {
                condition.evaluate(row).unwrap_or(false)
            });

            if matches {
                affected_count += 1;
            }
        }

        Ok(QueryResult::new(
            QueryType::Delete,
            table.name.clone(),
            0,
        ).with_affected_rows(affected_count))
    }

    async fn execute_count(&self, table: &Table, query: &Query) -> Result<QueryResult> {
        let mut count = 0;

        for row in &table.rows {
            let matches = query.conditions.iter().all(|condition| {
                condition.evaluate(row).unwrap_or(false)
            });

            if matches {
                count += 1;
            }
        }

        Ok(QueryResult::new(
            QueryType::Count,
            table.name.clone(),
            0,
        ).with_count(count))
    }

    fn sort_rows(&self, rows: &mut Vec<Row>, order_by: &[OrderBy]) {
        rows.sort_by(|a, b| {
            for order in order_by {
                let a_val = a.get(&order.column);
                let b_val = b.get(&order.column);

                let comparison = match (a_val, b_val) {
                    (Some(Value::Integer(a)), Some(Value::Integer(b))) => a.cmp(b),
                    (Some(Value::Text(a)), Some(Value::Text(b))) => a.cmp(b),
                    (Some(Value::Boolean(a)), Some(Value::Boolean(b))) => a.cmp(b),
                    (Some(Value::Float(a)), Some(Value::Float(b))) => {
                        a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    (Some(Value::Date(a)), Some(Value::Date(b))) => a.cmp(b),
                    (Some(Value::Time(a)), Some(Value::Time(b))) => a.cmp(b),
                    (Some(Value::DateTime(a)), Some(Value::DateTime(b))) => a.cmp(b),
                    (None, None) => std::cmp::Ordering::Equal,
                    (None, Some(_)) => std::cmp::Ordering::Less,
                    (Some(_), None) => std::cmp::Ordering::Greater,
                    _ => std::cmp::Ordering::Equal,
                };

                if comparison != std::cmp::Ordering::Equal {
                    return if order.ascending {
                        comparison
                    } else {
                        comparison.reverse()
                    };
                }
            }
            std::cmp::Ordering::Equal
        });
    }
}

impl Default for QueryEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// 查询构建器
pub struct QueryBuilder {
    query: Query,
}

impl QueryBuilder {
    pub fn new(table_name: &str) -> Self {
        Self {
            query: Query::select(table_name),
        }
    }

    pub fn select(table_name: &str) -> Self {
        Self::new(table_name)
    }

    pub fn insert(table_name: &str, data: HashMap<String, Value>) -> Self {
        Self {
            query: Query::insert(table_name, data),
        }
    }

    pub fn update(table_name: &str, data: HashMap<String, Value>) -> Self {
        Self {
            query: Query::update(table_name, data),
        }
    }

    pub fn delete(table_name: &str) -> Self {
        Self {
            query: Query::delete(table_name),
        }
    }

    pub fn count(table_name: &str) -> Self {
        Self {
            query: Query::count(table_name),
        }
    }

    pub fn where_condition(mut self, column: &str, operator: ComparisonOperator, value: Value) -> Self {
        self.query.conditions.push(Condition::new(column, operator, value));
        self
    }

    pub fn order_by(mut self, column: &str, ascending: bool) -> Self {
        self.query.order_by.push(OrderBy::new(column, ascending));
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.query.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.query.offset = Some(offset);
        self
    }

    pub fn build(self) -> Query {
        self.query
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ColumnDefinition, Schema, DataType};

    #[test]
    fn test_condition_evaluation() {
        let mut row = Row::new();
        row.set("age", Value::Integer(25));
        row.set("name", Value::Text("Alice".to_string()));

        let condition = Condition::new("age", ComparisonOperator::Equal, Value::Integer(25));
        assert!(condition.evaluate(&row).unwrap());

        let condition = Condition::new("name", ComparisonOperator::Like, Value::Text("A%".to_string()));
        assert!(condition.evaluate(&row).unwrap());
    }

    #[test]
    fn test_query_builder() {
        let query = QueryBuilder::select("users")
            .where_condition("age", ComparisonOperator::GreaterThan, Value::Integer(18))
            .order_by("name", true)
            .limit(10)
            .build();

        assert_eq!(query.table_name, "users");
        assert_eq!(query.conditions.len(), 1);
        assert_eq!(query.order_by.len(), 1);
        assert_eq!(query.limit, Some(10));
    }

    #[tokio::test]
    async fn test_query_execution() {
        let schema = Schema::new(vec![
            ColumnDefinition::new("id", DataType::Integer, true),
            ColumnDefinition::new("name", DataType::Text, false),
        ]);

        let mut table = Table::new("users".to_string(), schema);

        let mut row1 = Row::new();
        row1.set("id", Value::Integer(1));
        row1.set("name", Value::Text("Alice".to_string()));
        table.rows.push(row1);

        let mut row2 = Row::new();
        row2.set("id", Value::Integer(2));
        row2.set("name", Value::Text("Bob".to_string()));
        table.rows.push(row2);

        let query = QueryBuilder::select("users")
            .where_condition("name", ComparisonOperator::Equal, Value::Text("Alice".to_string()))
            .build();

        let engine = QueryEngine::new();
        let result = engine.execute(table, query).await.unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].get("name"), Some(&Value::Text("Alice".to_string())));
    }

    #[test]
    fn test_like_condition() {
        let mut row = Row::new();
        row.set("name", Value::Text("Alice Smith".to_string()));

        let condition = Condition::new("name", ComparisonOperator::Like, Value::Text("A%".to_string()));
        assert!(condition.evaluate(&row).unwrap());

        let condition = Condition::new("name", ComparisonOperator::Like, Value::Text("%Smith".to_string()));
        assert!(condition.evaluate(&row).unwrap());

        let condition = Condition::new("name", ComparisonOperator::Like, Value::Text("%lice%".to_string()));
        assert!(condition.evaluate(&row).unwrap());
    }
}