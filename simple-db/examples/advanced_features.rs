//! 高级特性示例
//!
//! 这个示例展示了Simple DB的高级功能，包括：
//! - 复杂查询
//! - 事务处理
//! - 备份和恢复
//! - 性能测试

use simple_db::engine::DatabaseEngine;
use simple_db::query::{QueryBuilder, ComparisonOperator};
use simple_db::types::{Value, DataType, Schema, ColumnDefinition};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Simple DB 高级特性示例");
    println!("{}", "=".repeat(40));
    println!();

    // 创建数据库引擎
    let mut engine = DatabaseEngine::new();

    // 设置不自动保存，以便控制何时保存
    engine.set_auto_save(false);

    // 1. 创建复杂的表结构
    println!("1. 创建复杂表结构...");
    create_complex_tables(&mut engine).await?;

    // 2. 批量插入大量数据
    println!("\n2. 批量插入大量数据...");
    insert_large_dataset(&mut engine).await?;

    // 3. 复杂查询示例
    println!("\n3. 复杂查询示例...");
    demonstrate_complex_queries(&engine).await?;

    // 4. 事务处理示例
    println!("\n4. 事务处理示例...");
    demonstrate_transactions(&mut engine).await?;

    // 5. 备份和恢复
    println!("\n5. 备份和恢复示例...");
    demonstrate_backup_restore(&mut engine).await?;

    // 6. 性能测试
    println!("\n6. 性能测试...");
    performance_tests(&mut engine).await?;

    // 7. 错误处理示例
    println!("\n7. 错误处理示例...");
    demonstrate_error_handling(&mut engine).await?;

    println!("\n高级特性示例完成！");
    Ok(())
}

/// 创建复杂的表结构
async fn create_complex_tables(engine: &mut DatabaseEngine) -> Result<(), Box<dyn std::error::Error>> {
    // 员工表
    let employee_schema = Schema::new(vec![
        ColumnDefinition::new("id", DataType::Integer, true),
        ColumnDefinition::new("name", DataType::Text, false),
        ColumnDefinition::new("department", DataType::Text, false),
        ColumnDefinition::new("salary", DataType::Float, false),
        ColumnDefinition::new("hire_date", DataType::Date, false),
        ColumnDefinition::new("is_manager", DataType::Boolean, false)
            .default_value(Value::Boolean(false)),
    ]);
    engine.create_table("employees", employee_schema).await?;
    println!("✓ 创建员工表");

    // 部门表
    let department_schema = Schema::new(vec![
        ColumnDefinition::new("id", DataType::Integer, true),
        ColumnDefinition::new("name", DataType::Text, false),
        ColumnDefinition::new("location", DataType::Text, false),
        ColumnDefinition::new("budget", DataType::Float, false),
    ]);
    engine.create_table("departments", department_schema).await?;
    println!("✓ 创建部门表");

    // 项目表
    let project_schema = Schema::new(vec![
        ColumnDefinition::new("id", DataType::Integer, true),
        ColumnDefinition::new("name", DataType::Text, false),
        ColumnDefinition::new("start_date", DataType::Date, false),
        ColumnDefinition::new("end_date", DataType::Date, true),
        ColumnDefinition::new("budget", DataType::Float, false),
        ColumnDefinition::new("status", DataType::Text, false),
    ]);
    engine.create_table("projects", project_schema).await?;
    println!("✓ 创建项目表");

    // 员工项目关联表
    let assignment_schema = Schema::new(vec![
        ColumnDefinition::new("employee_id", DataType::Integer, true),
        ColumnDefinition::new("project_id", DataType::Integer, true),
        ColumnDefinition::new("role", DataType::Text, false),
        ColumnDefinition::new("hours_per_week", DataType::Integer, false),
    ]);
    engine.create_table("assignments", assignment_schema).await?;
    println!("✓ 创建项目分配表");

    Ok(())
}

/// 插入大量数据
async fn insert_large_dataset(engine: &mut DatabaseEngine) -> Result<(), Box<dyn std::error::Error>> {
    let departments = vec![
        ("研发部", "北京", 1000000.0),
        ("市场部", "上海", 500000.0),
        ("销售部", "广州", 800000.0),
        ("人事部", "深圳", 300000.0),
    ];

    let mut department_data = Vec::new();
    for (i, (name, location, budget)) in departments.iter().enumerate() {
        let mut data = std::collections::HashMap::new();
        data.insert("id".to_string(), Value::Integer(i as i64 + 1));
        data.insert("name".to_string(), Value::Text(name.to_string()));
        data.insert("location".to_string(), Value::Text(location.to_string()));
        data.insert("budget".to_string(), Value::Float(*budget));
        department_data.push(data);
    }

    engine.batch_insert("departments", department_data).await?;
    println!("✓ 插入 {} 个部门", departments.len());

    // 插入员工数据
    let employees = vec![
        (1, "张三", "研发部", 15000.0, "2020-01-15", true),
        (2, "李四", "研发部", 12000.0, "2020-03-20", false),
        (3, "王五", "市场部", 10000.0, "2020-02-10", true),
        (4, "赵六", "销售部", 8000.0, "2020-04-05", false),
        (5, "钱七", "人事部", 9000.0, "2020-05-12", true),
        (6, "孙八", "研发部", 13000.0, "2020-06-18", false),
        (7, "周九", "市场部", 9500.0, "2020-07-22", false),
        (8, "吴十", "销售部", 8500.0, "2020-08-30", false),
    ];

    let mut employee_data = Vec::new();
    for (id, name, department, salary, hire_date, is_manager) in &employees {
        let mut data = std::collections::HashMap::new();
        data.insert("id".to_string(), Value::Integer(id));
        data.insert("name".to_string(), Value::Text(name.to_string()));
        data.insert("department".to_string(), Value::Text(department.to_string()));
        data.insert("salary".to_string(), Value::Float(salary));
        data.insert("hire_date".to_string(), Value::Text(hire_date.to_string()));
        data.insert("is_manager".to_string(), Value::Boolean(is_manager));
        employee_data.push(data);
    }

    engine.batch_insert("employees", employee_data).await?;
    println!("✓ 插入 {} 个员工", employees.len());

    // 插入项目数据
    let projects = vec![
        (1, "网站重构", "2021-01-01", "2021-06-30", 500000.0, "已完成"),
        (2, "移动应用开发", "2021-03-15", "2021-12-31", 800000.0, "进行中"),
        (3, "数据分析平台", "2021-05-01", "2022-03-31", 1200000.0, "计划中"),
    ];

    let mut project_data = Vec::new();
    for (id, name, start_date, end_date, budget, status) in &projects {
        let mut data = std::collections::HashMap::new();
        data.insert("id".to_string(), Value::Integer(id));
        data.insert("name".to_string(), Value::Text(name.to_string()));
        data.insert("start_date".to_string(), Value::Text(start_date.to_string()));
        data.insert("end_date".to_string(), Value::Text(end_date.to_string()));
        data.insert("budget".to_string(), Value::Float(budget));
        data.insert("status".to_string(), Value::Text(status.to_string()));
        project_data.push(data);
    }

    engine.batch_insert("projects", project_data).await?;
    println!("✓ 插入 {} 个项目", projects.len());

    // 插入项目分配数据
    let assignments = vec![
        (1, 1, "开发工程师", 40),
        (2, 1, "测试工程师", 35),
        (3, 2, "产品经理", 30),
        (1, 2, "技术负责人", 20),
        (5, 3, "项目经理", 25),
    ];

    let mut assignment_data = Vec::new();
    for (emp_id, proj_id, role, hours) in &assignments {
        let mut data = std::collections::HashMap::new();
        data.insert("employee_id".to_string(), Value::Integer(emp_id));
        data.insert("project_id".to_string(), Value::Integer(proj_id));
        data.insert("role".to_string(), Value::Text(role.to_string()));
        data.insert("hours_per_week".to_string(), Value::Integer(hours));
        assignment_data.push(data);
    }

    engine.batch_insert("assignments", assignment_data).await?;
    println!("✓ 插入 {} 个项目分配", assignments.len());

    Ok(())
}

/// 演示复杂查询
async fn demonstrate_complex_queries(engine: &DatabaseEngine) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n3.1 查询工资高于平均水平的员工:");
    let query = QueryBuilder::select("employees")
        .where_condition("salary", ComparisonOperator::GreaterThan, Value::Integer(11000))
        .order_by("salary", false)
        .build();
    let result = engine.query(query).await?;
    print_query_result(&result);

    println!("\n3.2 查询各部门的经理:");
    let query = QueryBuilder::select("employees")
        .where_condition("is_manager", ComparisonOperator::Equal, Value::Boolean(true))
        .order_by("department", true)
        .build();
    let result = engine.query(query).await?;
    print_query_result(&result);

    println!("\n3.3 查询预算超过500000的部门:");
    let query = QueryBuilder::select("departments")
        .where_condition("budget", ComparisonOperator::GreaterThan, Value::Float(500000.0))
        .order_by("budget", false)
        .build();
    let result = engine.query(query).await?;
    print_query_result(&result);

    println!("\n3.4 查询进行中的项目:");
    let query = QueryBuilder::select("projects")
        .where_condition("status", ComparisonOperator::Equal, Value::Text("进行中".to_string()))
        .build();
    let result = engine.query(query).await?;
    print_query_result(&result);

    println!("\n3.5 分页查询员工（每页3条）:");
    for page in 0..3 {
        println!("  第{}页:", page + 1);
        let query = QueryBuilder::select("employees")
            .limit(3)
            .offset(page * 3)
            .order_by("id", true)
            .build();
        let result = engine.query(query).await?;
        print_query_result(&result);
        if result.rows.len() < 3 {
            break;
        }
    }

    Ok(())
}

/// 演示事务处理
async fn demonstrate_transactions(engine: &mut DatabaseEngine) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n4.1 演示正常事务:");
    let result = engine.transaction(|tx| {
        // 创建新表
        let schema = Schema::new(vec![
            ColumnDefinition::new("id", DataType::Integer, true),
            ColumnDefinition::new("data", DataType::Text, false),
        ]);
        tx.create_table("transaction_test", schema)?;

        // 插入数据
        let mut data1 = std::collections::HashMap::new();
        data1.insert("id".to_string(), Value::Integer(1));
        data1.insert("data".to_string(), Value::Text("测试数据1".to_string()));
        tx.insert("transaction_test", data1)?;

        let mut data2 = std::collections::HashMap::new();
        data2.insert("id".to_string(), Value::Integer(2));
        data2.insert("data".to_string(), Value::Text("测试数据2".to_string()));
        tx.insert("transaction_test", data2)?;

        println!("  事务内操作完成");
        Ok::<(), simple_db::error::DatabaseError>(())
    }).await;

    match result {
        Ok(_) => {
            println!("✓ 事务提交成功");
            // 验证数据
            let query = QueryBuilder::select("transaction_test").build();
            let result = engine.query(query).await?;
            print_query_result(&result);
        }
        Err(e) => {
            println!("✗ 事务失败: {}", e);
        }
    }

    println!("\n4.2 演示事务回滚:");
    let result = engine.transaction(|tx| {
        // 尝试插入重复ID（应该失败）
        let mut data = std::collections::HashMap::new();
        data.insert("id".to_string(), Value::Integer(1)); // 重复ID
        data.insert("data".to_string(), Value::Text("重复数据".to_string()));
        tx.insert("transaction_test", data)?;

        Ok::<(), simple_db::error::DatabaseError>(())
    }).await;

    match result {
        Ok(_) => {
            println!("✓ 事务意外成功");
        }
        Err(e) => {
            println!("✓ 事务按预期失败（数据一致性得到保证）: {}", e);
        }
    }

    // 验证数据没有被破坏
    println!("\n4.3 验证数据一致性:");
    let query = QueryBuilder::count("transaction_test").build();
    let result = engine.query(query).await?;
    if let Some(count) = result.count {
        println!("  表中仍有 {} 行数据（没有被破坏）", count);
    }

    Ok(())
}

/// 演示备份和恢复
async fn demonstrate_backup_restore(engine: &mut DatabaseEngine) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n5.1 创建备份...");

    // 先获取当前表数
    let tables_before = engine.list_tables().await.len();
    println!("  备份前表数: {}", tables_before);

    // 创建备份
    let backup_path = "backup_test";
    engine.backup(backup_path).await?;
    println!("✓ 备份创建成功");

    // 删除一些表来模拟数据丢失
    println!("\n5.2 模拟数据丢失...");
    engine.drop_table("transaction_test").await?;
    engine.drop_table("projects").await?;
    println!("  删除了一些表");

    let tables_after_deletion = engine.list_tables().await.len();
    println!("  删除后表数: {}", tables_after_deletion);

    // 恢复备份
    println!("\n5.3 恢复备份...");
    engine.restore(backup_path).await?;
    println!("✓ 备份恢复成功");

    // 验证恢复结果
    let tables_after_restore = engine.list_tables().await.len();
    println!("  恢复后表数: {}", tables_after_restore);

    if tables_after_restore > tables_after_deletion {
        println!("✓ 数据恢复成功，表数增加了");
    }

    // 清理备份文件
    std::fs::remove_dir_all(backup_path).ok();
    println!("✓ 清理了备份文件");

    Ok(())
}

/// 性能测试
async fn performance_tests(engine: &mut DatabaseEngine) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n6.1 创建性能测试表...");
    let schema = Schema::new(vec![
        ColumnDefinition::new("id", DataType::Integer, true),
        ColumnDefinition::new("name", DataType::Text, false),
        ColumnDefinition::new("value", DataType::Float, false),
        ColumnDefinition::new("category", DataType::Text, false),
        ColumnDefinition::new("timestamp", DataType::DateTime, false),
    ]);
    engine.create_table("perf_test", schema).await?;

    // 批量插入性能测试
    println!("\n6.2 批量插入性能测试...");
    let start = std::time::Instant::now();

    let batch_size = 1000;
    let mut batch_data = Vec::new();

    for i in 0..batch_size {
        let mut data = std::collections::HashMap::new();
        data.insert("id".to_string(), Value::Integer(i));
        data.insert("name".to_string(), Value::Text(format!("Item {}", i)));
        data.insert("value".to_string(), Value::Float(i as f64 * 1.5));
        data.insert("category".to_string(), Value::Text(if i % 2 == 0 { "A" } else { "B" }.to_string()));
        data.insert("timestamp".to_string(), Value::Text(chrono::Utc::now().to_rfc3339().to_string()));
        batch_data.push(data);
    }

    let ids = engine.batch_insert("perf_test", batch_data).await?;
    let insert_duration = start.elapsed();
    println!("✓ 插入 {} 条数据，耗时: {:?}", ids.len(), insert_duration);
    println!("  平均每条: {:.2}ms", insert_duration.as_millis() as f64 / ids.len() as f64);

    // 查询性能测试
    println!("\n6.3 查询性能测试...");
    let start = std::time::Instant::now();

    let query = QueryBuilder::select("perf_test")
        .where_condition("category", ComparisonOperator::Equal, Value::Text("A".to_string()))
        .build();

    let result = engine.query(query).await?;
    let query_duration = start.elapsed();

    println!("✓ 查询 {} 条数据，耗时: {:?}", result.rows.len(), query_duration);
    println!("  平均每条: {:.2}ms", query_duration.as_millis() as f64 / result.rows.len() as f64);

    // 排序性能测试
    println!("\n6.4 排序性能测试...");
    let start = std::time::Instant::now();

    let query = QueryBuilder::select("perf_test")
        .order_by("value", false)
        .limit(100)
        .build();

    let result = engine.query(query).await?;
    let sort_duration = start.elapsed();

    println!("✓ 排序查询 {} 条数据，耗时: {:?}", result.rows.len(), sort_duration);

    // 统计性能测试
    println!("\n6.5 统计性能测试...");
    let start = std::time::Instant::now();

    let query = QueryBuilder::count("perf_test").build();
    let result = engine.query(query).await?;
    let count_duration = start.elapsed();

    if let Some(count) = result.count {
        println!("✓ 统计 {} 条数据，耗时: {:?}", count, count_duration);
    }

    // 清理测试表
    engine.drop_table("perf_test").await?;
    println!("\n✓ 清理了性能测试表");

    Ok(())
}

/// 演示错误处理
async fn demonstrate_error_handling(engine: &mut DatabaseEngine) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n7.1 测试重复创建表:");
    let schema = Schema::new(vec![
        ColumnDefinition::new("id", DataType::Integer, true),
    ]);

    match engine.create_table("employees", schema).await {
        Ok(_) => {
            println!("✗ 意外成功");
        }
        Err(e) => {
            println!("✓ 正确捕获错误: {}", e);
        }
    }

    println!("\n7.2 测试插入重复数据:");
    let mut data = std::collections::HashMap::new();
    data.insert("id".to_string(), Value::Integer(1));
    data.insert("name".to_string(), Value::Text("重复员工".to_string()));
    data.insert("department".to_string(), Value::Text("测试部".to_string()));
    data.insert("salary".to_string(), Value::Float(5000.0));
    data.insert("hire_date".to_string(), Value::Text("2021-01-01".to_string()));

    match engine.insert("employees", data).await {
        Ok(_) => {
            println!("✗ 意外成功");
        }
        Err(e) => {
            println!("✓ 正确捕获错误: {}", e);
        }
    }

    println!("\n7.3 测试查询不存在的表:");
    let query = QueryBuilder::select("nonexistent_table").build();

    match engine.query(query).await {
        Ok(_) => {
            println!("✗ 意外成功");
        }
        Err(e) => {
            println!("✓ 正确捕获错误: {}", e);
        }
    }

    println!("\n7.4 测试违反约束的操作:");
    let conditions = vec![("id".to_string(), ComparisonOperator::Equal, Value::Integer(999))];
    let mut updates = std::collections::HashMap::new();
    updates.insert("name".to_string(), Value::Text("测试".to_string()));

    match engine.update("employees", conditions, updates).await {
        Ok(_) => {
            println!("✓ 更新成功（受影响行数为0）");
        }
        Err(e) => {
            println!("✓ 正确捕获错误: {}", e);
        }
    }

    println!("\n7.5 测试类型不匹配:");
    let mut data = std::collections::HashMap::new();
    data.insert("id".to_string(), Value::Text("不是数字".to_string())); // 错误的类型
    data.insert("name".to_string(), Value::Text("测试".to_string()));

    match engine.insert("departments", data).await {
        Ok(_) => {
            println!("✗ 意外成功");
        }
        Err(e) => {
            println!("✓ 正确捕获错误: {}", e);
        }
    }

    Ok(())
}

/// 打印查询结果（简化版）
fn print_query_result(result: &simple_db::query::QueryResult) {
    if result.rows.is_empty() {
        println!("  (没有数据)");
        return;
    }

    // 只打印前几行，避免输出过长
    let max_rows = 5;
    let rows_to_print = if result.rows.len() > max_rows {
        &result.rows[..max_rows]
    } else {
        &result.rows
    };

    for row in rows_to_print {
        print!("  | ");
        for col in ["id", "name", "department", "salary", "status"] {
            if let Some(value) = row.get(col) {
                print!("{}: {} | ", col, value.to_string());
            }
        }
        println!();
    }

    if result.rows.len() > max_rows {
        println!("  ... (还有 {} 行)", result.rows.len() - max_rows);
    }

    println!("  查询耗时: {} ms", result.execution_time_ms);
}