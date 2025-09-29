//! 基本使用示例
//!
//! 这个示例展示了如何使用Simple DB进行基本的数据库操作

use simple_db::engine::DatabaseEngine;
use simple_db::query::{QueryBuilder, ComparisonOperator};
use simple_db::types::{Value, DataType, Schema, ColumnDefinition};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Simple DB 基本使用示例");
    println!("{}", "=".repeat(40));
    println!();

    // 创建数据库引擎
    let mut engine = DatabaseEngine::new();

    // 1. 创建表
    println!("1. 创建用户表...");
    let schema = Schema::new(vec![
        ColumnDefinition::new("id", DataType::Integer, true),
        ColumnDefinition::new("name", DataType::Text, false),
        ColumnDefinition::new("age", DataType::Integer, false),
        ColumnDefinition::new("email", DataType::Text, false).unique(true),
        ColumnDefinition::new("active", DataType::Boolean, false)
            .default_value(Value::Boolean(true)),
    ]);

    engine.create_table("users", schema).await?;
    println!("✓ 用户表创建成功");

    // 2. 插入数据
    println!("\n2. 插入用户数据...");
    let users = vec![
        vec![
            ("id", Value::Integer(1)),
            ("name", Value::Text("张三".to_string())),
            ("age", Value::Integer(25)),
            ("email", Value::Text("zhangsan@example.com".to_string())),
        ],
        vec![
            ("id", Value::Integer(2)),
            ("name", Value::Text("李四".to_string())),
            ("age", Value::Integer(30)),
            ("email", Value::Text("lisi@example.com".to_string())),
        ],
        vec![
            ("id", Value::Integer(3)),
            ("name", Value::Text("王五".to_string())),
            ("age", Value::Integer(28)),
            ("email", Value::Text("wangwu@example.com".to_string())),
        ],
    ];

    for user_data in users {
        let data: std::collections::HashMap<String, Value> = user_data
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        let id = engine.insert("users", data).await?;
        println!("✓ 插入用户，ID: {}", id);
    }

    // 3. 查询所有用户
    println!("\n3. 查询所有用户:");
    let query = QueryBuilder::select("users").build();
    let result = engine.query(query).await?;
    print_query_result(&result);

    // 4. 条件查询
    println!("\n4. 查询年龄大于25的用户:");
    let query = QueryBuilder::select("users")
        .where_condition("age", ComparisonOperator::GreaterThan, Value::Integer(25))
        .build();
    let result = engine.query(query).await?;
    print_query_result(&result);

    // 5. 排序查询
    println!("\n5. 按年龄降序查询用户:");
    let query = QueryBuilder::select("users")
        .order_by("age", false)
        .build();
    let result = engine.query(query).await?;
    print_query_result(&result);

    // 6. 分页查询
    println!("\n6. 分页查询用户 (每页2条，第1页):");
    let query = QueryBuilder::select("users")
        .limit(2)
        .offset(0)
        .build();
    let result = engine.query(query).await?;
    print_query_result(&result);

    // 7. 更新数据
    println!("\n7. 更新用户数据...");
    let conditions = vec![("id".to_string(), ComparisonOperator::Equal, Value::Integer(1))];
    let mut updates = std::collections::HashMap::new();
    updates.insert("age".to_string(), Value::Integer(26));
    updates.insert("active".to_string(), Value::Boolean(false));

    let affected = engine.update("users", conditions, updates).await?;
    println!("✓ 更新了 {} 行", affected);

    // 8. 删除数据
    println!("\n8. 删除用户数据...");
    let conditions = vec![("id".to_string(), ComparisonOperator::Equal, Value::Integer(3))];
    let affected = engine.delete("users", conditions).await?;
    println!("✓ 删除了 {} 行", affected);

    // 9. 查询更新后的数据
    println!("\n9. 更新后的用户数据:");
    let query = QueryBuilder::select("users").build();
    let result = engine.query(query).await?;
    print_query_result(&result);

    // 10. 统计查询
    println!("\n10. 统计用户数量:");
    let query = QueryBuilder::count("users").build();
    let result = engine.query(query).await?;
    if let Some(count) = result.count {
        println!("✓ 用户总数: {}", count);
    }

    // 11. 表信息
    println!("\n11. 表信息:");
    let table_info = engine.get_table_info("users").await?;
    println!("表名: {}", table_info.name);
    println!("行数: {}", table_info.row_count);
    println!("创建时间: {}", table_info.created_at);
    println!("列信息:");
    for column in &table_info.schema.columns {
        let nullable = if column.nullable { "YES" } else { "NO" };
        let unique = if column.unique { "YES" } else { "NO" };
        let primary = if column.primary_key { "YES" } else { "NO" };
        println!("  {}: {} NULL={}, UNIQUE={}, PK={}",
            column.name, column.data_type.to_string(), nullable, unique, primary);
    }

    // 12. 数据库统计
    println!("\n12. 数据库统计信息:");
    let stats = engine.get_stats().await?;
    println!("总表数: {}", stats.total_tables);
    println!("总行数: {}", stats.total_rows);
    println!("存储大小: {} 字节", stats.storage_stats.total_size());

    // 13. 事务示例
    println!("\n13. 事务示例...");
    engine.set_auto_save(false); // 禁用自动保存避免死锁
    let result = engine.transaction(|tx| {
        // 在事务中创建新表
        let schema = Schema::new(vec![
            ColumnDefinition::new("id", DataType::Integer, true),
            ColumnDefinition::new("product_name", DataType::Text, false),
            ColumnDefinition::new("price", DataType::Float, false),
        ]);
        tx.create_table("products", schema)?;

        // 在事务中插入数据
        let mut data = std::collections::HashMap::new();
        data.insert("id".to_string(), Value::Integer(1));
        data.insert("product_name".to_string(), Value::Text("Laptop".to_string()));
        data.insert("price".to_string(), Value::Float(999.99));
        tx.insert("products", data)?;

        Ok::<(), simple_db::error::DatabaseError>(())
    }).await;

    match result {
        Ok(_) => {
            println!("✓ 事务执行成功");
            // 查询产品表
            let query = QueryBuilder::select("products").build();
            let result = engine.query(query).await?;
            print_query_result(&result);
        }
        Err(e) => {
            println!("✗ 事务执行失败: {}", e);
        }
    }

    // 14. 批量操作
    println!("\n14. 批量插入示例...");
    let mut batch_data = Vec::new();
    for i in 100..105 {
        let mut data = std::collections::HashMap::new();
        data.insert("id".to_string(), Value::Integer(i));
        data.insert("name".to_string(), Value::Text(format!("用户{}", i)));
        data.insert("age".to_string(), Value::Integer(20 + (i % 10)));
        data.insert("email".to_string(), Value::Text(format!("user{}@example.com", i)));
        batch_data.push(data);
    }

    let ids = engine.batch_insert("users", batch_data).await?;
    println!("✓ 批量插入了 {} 个用户", ids.len());

    // 查询最终结果
    println!("\n15. 最终用户数据:");
    let query = QueryBuilder::select("users")
        .order_by("id", true)
        .build();
    let result = engine.query(query).await?;
    print_query_result(&result);

    // 16. 保存到磁盘
    println!("\n16. 保存数据库到磁盘...");
    engine.save_to_disk().await?;
    println!("✓ 数据库保存成功");

    println!("\n示例完成！");
    Ok(())
}

/// 打印查询结果
fn print_query_result(result: &simple_db::query::QueryResult) {
    if result.rows.is_empty() {
        println!("(没有数据)");
        return;
    }

    // 获取所有列名并排序
    let mut columns: Vec<String> = Vec::new();
    if let Some(first_row) = result.rows.first() {
        columns = first_row.columns().into_iter().map(|s| s.to_string()).collect();
        columns.sort();
    }

    // 计算每列宽度
    let mut widths = std::collections::HashMap::new();
    for col in &columns {
        let mut max_width = col.len();
        for row in &result.rows {
            if let Some(value) = row.get(col) {
                let value_str = value.to_string();
                if value_str.len() > max_width {
                    max_width = value_str.len();
                }
            }
        }
        widths.insert(col.clone(), max_width);
    }

    // 打印分隔线
    print!("+");
    for col in &columns {
        print!("{:-<width$}-+", "", width = widths[col] + 2);
    }
    println!();

    // 打印表头
    print!("|");
    for col in &columns {
        print!(" {:<width$} |", col, width = widths[col]);
    }
    println!();

    // 打印分隔线
    print!("+");
    for col in &columns {
        print!("{:-<width$}-+", "", width = widths[col] + 2);
    }
    println!();

    // 打印数据
    for row in &result.rows {
        print!("|");
        for col in &columns {
            let value = row.get(col)
                .map(|v| v.to_string())
                .unwrap_or_else(|| "NULL".to_string());
            print!(" {:<width$} |", value, width = widths[col]);
        }
        println!();
    }

    // 打印分隔线
    print!("+");
    for col in &columns {
        print!("{:-<width$}-+", "", width = widths[col] + 2);
    }
    println!();

    println!("查询耗时: {} ms, 返回 {} 行", result.execution_time_ms, result.rows.len());
}