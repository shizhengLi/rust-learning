use clap::{Parser, Subcommand};
use rustyline::Editor;
use std::collections::HashMap;
use tokio;

use simple_db::engine::DatabaseEngine;
use simple_db::query::{QueryBuilder, ComparisonOperator};
use simple_db::types::{Value, DataType, Schema, ColumnDefinition};

/// Simple DB - 一个简单的内存数据库
#[derive(Parser, Debug)]
#[command(name = "simple-db")]
#[command(about = "一个简单的内存数据库", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// 数据库文件路径
    #[arg(short, long)]
    database: Option<String>,

    /// 启用调试模式
    #[arg(long)]
    debug: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 交互式模式
    Shell,
    /// 执行SQL文件
    Execute {
        /// SQL文件路径
        file: String,
    },
    /// 运行示例
    Example,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // 初始化数据库引擎
    let mut engine = DatabaseEngine::new();

    // 如果指定了数据库文件，尝试加载
    if let Some(db_path) = &args.database {
        println!("正在加载数据库: {}", db_path);
        match DatabaseEngine::load_from_disk().await {
            Ok(loaded_engine) => {
                engine = loaded_engine;
                println!("数据库加载成功");
            }
            Err(e) => {
                println!("警告: 无法加载数据库: {}", e);
                println!("将创建新的数据库");
            }
        }
    }

    // 根据命令执行不同操作
    match args.command {
        Some(Commands::Shell) => {
            run_interactive_shell(engine).await;
        }
        Some(Commands::Execute { file }) => {
            execute_sql_file(&mut engine, &file).await?;
        }
        Some(Commands::Example) => {
            run_example(&engine).await;
        }
        None => {
            println!("Simple DB - 简单的内存数据库");
            println!("使用 --help 查看帮助");
            println!();
            println!("可用命令:");
            println!("  simple-db shell     # 启动交互式Shell");
            println!("  simple-db example   # 运行示例");
            println!("  simple-db execute -f file.sql  # 执行SQL文件");
        }
    }

    Ok(())
}

/// 运行交互式Shell
async fn run_interactive_shell(mut engine: DatabaseEngine) {
    println!("Simple DB 交互式Shell");
    println!("输入 'help' 查看帮助，'exit' 退出");
    println!();

    let mut rl = Editor::<()>::new().expect("Failed to create readline editor");
    let mut current_db = None;

    loop {
        let readline = rl.readline(&format!("{}> ", current_db.as_deref().unwrap_or("nodb")));
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                // 添加历史记录
                rl.add_history_entry(line);

                match handle_command(&mut engine, line, &mut current_db).await {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("错误: {}", e);
                    }
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("使用 'exit' 命令退出");
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("再见！");
                break;
            }
            Err(err) => {
                eprintln!("读取错误: {}", err);
                break;
            }
        }
    }
}

/// 处理用户命令
async fn handle_command(
    engine: &mut DatabaseEngine,
    command: &str,
    _current_db: &mut Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(());
    }

    match parts[0].to_lowercase().as_str() {
        "help" => {
            print_help();
        }
        "exit" | "quit" => {
            std::process::exit(0);
        }
        "tables" => {
            list_tables(engine).await;
        }
        "create" => {
            if parts.len() >= 3 && parts[1].to_lowercase() == "table" {
                let table_name = parts[2];
                create_table_interactive(engine, table_name).await?;
            } else {
                println!("用法: CREATE TABLE table_name");
            }
        }
        "drop" => {
            if parts.len() >= 3 && parts[1].to_lowercase() == "table" {
                let table_name = parts[2];
                drop_table(engine, table_name).await?;
            } else {
                println!("用法: DROP TABLE table_name");
            }
        }
        "insert" => {
            if parts.len() >= 3 && parts[1].to_lowercase() == "into" {
                let table_name = parts[2];
                insert_interactive(engine, table_name).await?;
            } else {
                println!("用法: INSERT INTO table_name");
            }
        }
        "select" => {
            if parts.len() >= 3 && parts[2].to_lowercase() == "*" && parts.len() >= 5 && parts[3].to_lowercase() == "from" {
                let table_name = parts[4];
                select_all(engine, table_name).await;
            } else {
                println!("用法: SELECT * FROM table_name");
            }
        }
        "update" => {
            if parts.len() >= 3 && parts[2].to_lowercase() == "set" {
                let table_name = parts[1];
                update_interactive(engine, table_name).await?;
            } else {
                println!("用法: UPDATE table_name SET ...");
            }
        }
        "delete" => {
            if parts.len() >= 3 && parts[1].to_lowercase() == "from" {
                let table_name = parts[2];
                delete_interactive(engine, table_name).await?;
            } else {
                println!("用法: DELETE FROM table_name");
            }
        }
        "describe" | "desc" => {
            if parts.len() >= 2 {
                let table_name = parts[1];
                describe_table(engine, table_name).await?;
            } else {
                println!("用法: DESCRIBE table_name");
            }
        }
        "count" => {
            if parts.len() >= 3 && parts[1].to_lowercase() == "from" {
                let table_name = parts[2];
                count_table(engine, table_name).await;
            } else {
                println!("用法: COUNT FROM table_name");
            }
        }
        "save" => {
            engine.save_to_disk().await?;
            println!("数据库已保存");
        }
        "load" => {
            match DatabaseEngine::load_from_disk().await {
                Ok(loaded_engine) => {
                    *engine = loaded_engine;
                    println!("数据库加载成功");
                }
                Err(e) => {
                    println!("加载失败: {}", e);
                }
            }
        }
        "stats" => {
            show_stats(engine).await;
        }
        "example" => {
            run_example(&engine).await;
        }
        "clear" => {
            print!("{}[2J{}[H", 27 as char, 27 as char);
        }
        _ => {
            println!("未知命令: '{}'. 输入 'help' 查看帮助", parts[0]);
        }
    }

    Ok(())
}

/// 打印帮助信息
fn print_help() {
    println!("可用命令:");
    println!("  help                    - 显示此帮助信息");
    println!("  exit/quit               - 退出程序");
    println!("  tables                  - 列出所有表");
    println!("  CREATE TABLE name       - 创建表");
    println!("  DROP TABLE name         - 删除表");
    println!("  INSERT INTO name        - 向表插入数据");
    println!("  SELECT * FROM name      - 查询表中的所有数据");
    println!("  UPDATE name SET ...     - 更新表数据");
    println!("  DELETE FROM name        - 删除表数据");
    println!("  DESCRIBE name           - 显示表结构");
    println!("  COUNT FROM name         - 统计表的行数");
    println!("  save                    - 保存数据库到磁盘");
    println!("  load                    - 从磁盘加载数据库");
    println!("  stats                   - 显示数据库统计信息");
    println!("  example                 - 运行示例");
    println!("  clear                   - 清屏");
}

/// 列出所有表
async fn list_tables(engine: &DatabaseEngine) {
    let tables = engine.list_tables().await;
    if tables.is_empty() {
        println!("没有表");
    } else {
        println!("表列表:");
        for table in &tables {
            println!("  - {} ({} 行)", table.name, table.row_count);
        }
    }
}

/// 交互式创建表
async fn create_table_interactive(engine: &mut DatabaseEngine, table_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{self, Write};

    println!("创建表: {}", table_name);
    println!("输入列定义 (格式: 名称 类型 [主键] [唯一] [非空] [默认值])");
    println!("输入空行结束");

    let mut columns = Vec::new();
    let mut primary_key_count = 0;

    loop {
        print!("列 {}: ", columns.len() + 1);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            break;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.len() < 2 {
            println!("错误: 至少需要名称和类型");
            continue;
        }

        let column_name = parts[0];
        let data_type = DataType::from_str(parts[1])?;

        let mut column_def = ColumnDefinition::new(column_name, data_type.clone(), false);

        for part in &parts[2..] {
            match part.to_lowercase().as_str() {
                "primary" | "pk" => {
                    column_def.primary_key = true;
                    column_def.unique = true;
                    column_def.nullable = false;
                    primary_key_count += 1;
                }
                "unique" => {
                    column_def.unique = true;
                }
                "not" | "null" => {
                    if part.to_lowercase() == "not" && parts.get(3).map(|s| s.to_lowercase()) == Some("null".to_string()) {
                        column_def.nullable = false;
                    }
                }
                "default" => {
                    if let Some(default_part) = parts.get(3) {
                        let default_value = parse_default_value(default_part, &data_type)?;
                        column_def.default_value = Some(default_value);
                    }
                }
                _ => {}
            }
        }

        columns.push(column_def);
    }

    if columns.is_empty() {
        println!("错误: 至少需要一个列");
        return Ok(());
    }

    if primary_key_count > 1 {
        println!("错误: 只能有一个主键");
        return Ok(());
    }

    let schema = Schema::new(columns);
    engine.create_table(table_name, schema).await?;
    println!("表 '{}' 创建成功", table_name);

    Ok(())
}

/// 解析默认值
fn parse_default_value(value: &str, data_type: &DataType) -> Result<Value, Box<dyn std::error::Error>> {
    match data_type {
        DataType::Integer => {
            let int_val: i64 = value.parse()?;
            Ok(Value::Integer(int_val))
        }
        DataType::Text => {
            Ok(Value::Text(value.to_string()))
        }
        DataType::Boolean => {
            let bool_val: bool = value.parse()?;
            Ok(Value::Boolean(bool_val))
        }
        DataType::Float => {
            let float_val: f64 = value.parse()?;
            Ok(Value::Float(float_val))
        }
        _ => {
            Ok(Value::Text(value.to_string()))
        }
    }
}

/// 交互式插入数据
async fn insert_interactive(engine: &mut DatabaseEngine, table_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{self, Write};

    // 获取表结构
    let table_info = engine.get_table_info(table_name).await?;

    println!("插入数据到表: {}", table_name);
    println!("表结构:");
    for column in &table_info.schema.columns {
        let nullable = if column.nullable { "NULL" } else { "NOT NULL" };
        let unique = if column.unique { " UNIQUE" } else { "" };
        let primary = if column.primary_key { " PRIMARY KEY" } else { "" };
        println!("  {}: {}{}{}{}", column.name, column.data_type.to_string(), nullable, unique, primary);
    }

    let mut data = HashMap::new();

    for column in &table_info.schema.columns {
        if column.primary_key {
            // 主键自动生成
            continue;
        }

        print!("{} ({}): ", column.name, column.data_type.to_string());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            if let Some(default) = &column.default_value {
                data.insert(column.name.clone(), default.clone());
            } else if !column.nullable {
                println!("错误: 列 '{}' 不能为空", column.name);
                return Err("列不能为空".into());
            }
        } else {
            let value = parse_value(input, &column.data_type)?;
            data.insert(column.name.clone(), value);
        }
    }

    let id = engine.insert(table_name, data).await?;
    println!("插入成功，ID: {}", id);

    Ok(())
}

/// 解析值
fn parse_value(value: &str, data_type: &DataType) -> Result<Value, Box<dyn std::error::Error>> {
    match data_type {
        DataType::Integer => {
            let int_val: i64 = value.parse()?;
            Ok(Value::Integer(int_val))
        }
        DataType::Text => {
            Ok(Value::Text(value.to_string()))
        }
        DataType::Boolean => {
            let bool_val = match value.to_lowercase().as_str() {
                "true" | "1" | "yes" | "y" => true,
                "false" | "0" | "no" | "n" => false,
                _ => return Err("无效的布尔值".into()),
            };
            Ok(Value::Boolean(bool_val))
        }
        DataType::Float => {
            let float_val: f64 = value.parse()?;
            Ok(Value::Float(float_val))
        }
        _ => {
            Ok(Value::Text(value.to_string()))
        }
    }
}

/// 查询所有数据
async fn select_all(engine: &DatabaseEngine, table_name: &str) {
    let query = QueryBuilder::select(table_name).build();

    match engine.query(query).await {
        Ok(result) => {
            if result.rows.is_empty() {
                println!("表 '{}' 中没有数据", table_name);
            } else {
                println!("表 '{}' 中的数据 ({} 行):", table_name, result.rows.len());
                print_table(&result.rows);
            }
        }
        Err(e) => {
            println!("查询失败: {}", e);
        }
    }
}

/// 格式化输出表格
fn print_table(rows: &[simple_db::types::Row]) {
    if rows.is_empty() {
        return;
    }

    // 获取所有列名
    let mut columns: Vec<String> = rows[0].columns().into_iter().map(|s| s.to_string()).collect();
    columns.sort(); // 按列名排序

    // 计算每列的最大宽度
    let mut widths = HashMap::new();
    for col in &columns {
        let mut max_width = col.len();
        for row in rows {
            if let Some(value) = row.get(col) {
                let value_str = value.to_string();
                if value_str.len() > max_width {
                    max_width = value_str.len();
                }
            }
        }
        widths.insert(col.clone(), max_width);
    }

    // 打印表头
    print!("+");
    for col in &columns {
        print!("{:-<width$}-+", "", width = widths[col] + 2);
    }
    println!();

    print!("|");
    for col in &columns {
        print!(" {:<width$} |", col, width = widths[col]);
    }
    println!();

    print!("+");
    for col in &columns {
        print!("{:-<width$}-+", "", width = widths[col] + 2);
    }
    println!();

    // 打印数据行
    for row in rows {
        print!("|");
        for col in &columns {
            let value = row.get(col).map(|v| v.to_string()).unwrap_or_else(|| "NULL".to_string());
            print!(" {:<width$} |", value, width = widths[col]);
        }
        println!();
    }

    // 打印表尾
    print!("+");
    for col in &columns {
        print!("{:-<width$}-+", "", width = widths[col] + 2);
    }
    println!();
}

/// 交互式更新数据
async fn update_interactive(engine: &mut DatabaseEngine, table_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{self, Write};

    println!("更新表: {}", table_name);
    print!("输入要更新的行ID: ");
    io::stdout().flush()?;

    let mut id_input = String::new();
    io::stdin().read_line(&mut id_input)?;
    let id_input = id_input.trim();

    let id: uuid::Uuid = id_input.parse()?;

    println!("输入要更新的列和值 (格式: 列名=值)");
    println!("输入空行结束");

    let mut updates = HashMap::new();

    loop {
        print!("更新 {}: ", updates.len() + 1);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            break;
        }

        if let Some((column, value_str)) = input.split_once('=') {
            let table_info = engine.get_table_info(table_name).await?;
            if let Some(column_def) = table_info.schema.get_column(column) {
                let value = parse_value(value_str, &column_def.data_type)?;
                updates.insert(column.to_string(), value);
            } else {
                println!("错误: 列 '{}' 不存在", column);
            }
        } else {
            println!("错误: 格式应为 '列名=值'");
        }
    }

    if updates.is_empty() {
        println!("没有要更新的列");
        return Ok(());
    }

    let conditions = vec![
        ("id".to_string(), ComparisonOperator::Equal, Value::Text(id.to_string()))
    ];

    let affected = engine.update(table_name, conditions, updates).await?;
    println!("更新了 {} 行", affected);

    Ok(())
}

/// 交互式删除数据
async fn delete_interactive(engine: &mut DatabaseEngine, table_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{self, Write};

    println!("从表 {} 删除数据", table_name);
    println!("1. 根据ID删除");
    println!("2. 根据条件删除");
    print!("选择删除方式: ");
    io::stdout().flush()?;

    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let choice = choice.trim();

    match choice {
        "1" => {
            print!("输入要删除的行ID: ");
            io::stdout().flush()?;

            let mut id_input = String::new();
            io::stdin().read_line(&mut id_input)?;
            let id_input = id_input.trim();

            let id: uuid::Uuid = id_input.parse()?;

            let conditions = vec![
                ("id".to_string(), ComparisonOperator::Equal, Value::Text(id.to_string()))
            ];

            let affected = engine.delete(table_name, conditions).await?;
            println!("删除了 {} 行", affected);
        }
        "2" => {
            println!("输入删除条件 (格式: 列名=值)");
            println!("输入空行结束");

            let mut conditions = Vec::new();

            loop {
                print!("条件 {}: ", conditions.len() + 1);
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let input = input.trim();

                if input.is_empty() {
                    break;
                }

                if let Some((column, value_str)) = input.split_once('=') {
                    let table_info = engine.get_table_info(table_name).await?;
                    if let Some(column_def) = table_info.schema.get_column(column) {
                        let value = parse_value(value_str, &column_def.data_type)?;
                        conditions.push((column.to_string(), ComparisonOperator::Equal, value));
                    } else {
                        println!("错误: 列 '{}' 不存在", column);
                    }
                } else {
                    println!("错误: 格式应为 '列名=值'");
                }
            }

            if conditions.is_empty() {
                println!("没有指定条件");
                return Ok(());
            }

            let affected = engine.delete(table_name, conditions).await?;
            println!("删除了 {} 行", affected);
        }
        _ => {
            println!("无效选择");
        }
    }

    Ok(())
}

/// 描述表结构
async fn describe_table(engine: &DatabaseEngine, table_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let table_info = engine.get_table_info(table_name).await?;

    println!("表: {}", table_info.name);
    println!("创建时间: {}", table_info.created_at);
    println!("行数: {}", table_info.row_count);
    println!();
    println!("列信息:");

    let mut max_name_len = 4;
    let mut max_type_len = 4;

    for column in &table_info.schema.columns {
        if column.name.len() > max_name_len {
            max_name_len = column.name.len();
        }
        let type_str = column.data_type.to_string();
        if type_str.len() > max_type_len {
            max_type_len = type_str.len();
        }
    }

    // 打印表头
    println!("{:<name_width$} | {:<type_width$} | NULL | UNIQUE | PK | DEFAULT",
             "名称", "类型", name_width = max_name_len, type_width = max_type_len);
    println!("{:-<name_width$}-+{:-<type_width$}-+------+--------+---+--------",
             "", "", name_width = max_name_len, type_width = max_type_len);

    // 打印列信息
    for column in &table_info.schema.columns {
        let null_str = if column.nullable { "YES" } else { "NO" };
        let unique_str = if column.unique { "YES" } else { "NO" };
        let pk_str = if column.primary_key { "YES" } else { "NO" };
        let default_str = column.default_value.as_ref()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "".to_string());

        println!("{:<name_width$} | {:<type_width$} | {:<4} | {:<6} | {:<2} | {}",
                 column.name, column.data_type.to_string(), null_str, unique_str, pk_str, default_str,
                 name_width = max_name_len, type_width = max_type_len);
    }

    Ok(())
}

/// 统计表行数
async fn count_table(engine: &DatabaseEngine, table_name: &str) {
    let query = QueryBuilder::count(table_name).build();

    match engine.query(query).await {
        Ok(result) => {
            if let Some(count) = result.count {
                println!("表 '{}' 共有 {} 行", table_name, count);
            } else {
                println!("表 '{}' 中没有数据", table_name);
            }
        }
        Err(e) => {
            println!("统计失败: {}", e);
        }
    }
}

/// 显示数据库统计信息
async fn show_stats(engine: &DatabaseEngine) {
    match engine.get_stats().await {
        Ok(stats) => {
            println!("数据库统计信息:");
            println!("  总表数: {}", stats.total_tables);
            println!("  总行数: {}", stats.total_rows);
            println!("  日志文件大小: {} 字节", stats.storage_stats.log_file_size);
            println!("  快照文件大小: {} 字节", stats.storage_stats.snapshot_file_size);
            println!("  总存储大小: {} 字节", stats.storage_stats.total_size());
            println!("  日志条目数: {}", stats.storage_stats.total_log_entries);
            println!("  当前日志ID: {}", stats.storage_stats.current_log_id);
        }
        Err(e) => {
            println!("获取统计信息失败: {}", e);
        }
    }
}

/// 删除表
async fn drop_table(engine: &mut DatabaseEngine, table_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{self, Write};

    print!("确定要删除表 '{}' 吗? (y/N): ", table_name);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    if input == "y" || input == "yes" {
        engine.drop_table(table_name).await?;
        println!("表 '{}' 已删除", table_name);
    } else {
        println!("操作已取消");
    }

    Ok(())
}

/// 执行SQL文件
async fn execute_sql_file(_engine: &mut DatabaseEngine, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(file_path)?;
    let statements: Vec<&str> = content.split(';').filter(|s| !s.trim().is_empty()).collect();

    println!("执行SQL文件: {}", file_path);
    println!("共 {} 条语句", statements.len());

    for (i, statement) in statements.iter().enumerate() {
        println!("执行语句 {}: {}", i + 1, statement.trim());
        // 这里可以扩展SQL解析和执行
        println!("(SQL执行功能待实现)");
    }

    Ok(())
}

/// 运行示例
async fn run_example(engine: &DatabaseEngine) {
    println!("运行Simple DB示例...");
    println!();

    // 创建示例表
    println!("1. 创建表...");
    let schema = Schema::new(vec![
        ColumnDefinition::new("id", DataType::Integer, true),
        ColumnDefinition::new("name", DataType::Text, false),
        ColumnDefinition::new("age", DataType::Integer, false),
        ColumnDefinition::new("email", DataType::Text, false).unique(true),
    ]);

    match engine.create_table("users", schema).await {
        Ok(_) => println!("✓ 创建表 'users' 成功"),
        Err(e) => println!("✗ 创建表失败: {}", e),
    }

    let schema = Schema::new(vec![
        ColumnDefinition::new("id", DataType::Integer, true),
        ColumnDefinition::new("title", DataType::Text, false),
        ColumnDefinition::new("content", DataType::Text, false),
        ColumnDefinition::new("author_id", DataType::Integer, false),
        ColumnDefinition::new("created_at", DataType::DateTime, false)
            .default_value(Value::Text(chrono::Utc::now().to_rfc3339().to_string())),
    ]);

    match engine.create_table("posts", schema).await {
        Ok(_) => println!("✓ 创建表 'posts' 成功"),
        Err(e) => println!("✗ 创建表失败: {}", e),
    }

    println!();

    // 插入示例数据
    println!("2. 插入示例数据...");

    let users = vec![
        vec![("id", Value::Integer(1)), ("name", Value::Text("Alice".to_string())), ("age", Value::Integer(28)), ("email", Value::Text("alice@example.com".to_string()))],
        vec![("id", Value::Integer(2)), ("name", Value::Text("Bob".to_string())), ("age", Value::Integer(32)), ("email", Value::Text("bob@example.com".to_string()))],
        vec![("id", Value::Integer(3)), ("name", Value::Text("Charlie".to_string())), ("age", Value::Integer(25)), ("email", Value::Text("charlie@example.com".to_string()))],
    ];

    for user_data in users {
        let data: HashMap<String, Value> = user_data.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
        match engine.insert("users", data).await {
            Ok(_) => println!("✓ 插入用户成功"),
            Err(e) => println!("✗ 插入用户失败: {}", e),
        }
    }

    let posts = vec![
        vec![("id", Value::Integer(1)), ("title", Value::Text("Hello World".to_string())), ("content", Value::Text("这是我的第一篇博客文章".to_string())), ("author_id", Value::Integer(1))],
        vec![("id", Value::Integer(2)), ("title", Value::Text("Rust编程".to_string())), ("content", Value::Text("Rust是一门很棒的系统编程语言".to_string())), ("author_id", Value::Integer(2))],
        vec![("id", Value::Integer(3)), ("title", Value::Text("数据库设计".to_string())), ("content", Value::Text("数据库设计是软件开发的重要部分".to_string())), ("author_id", Value::Integer(1))],
    ];

    for post_data in posts {
        let data: HashMap<String, Value> = post_data.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
        match engine.insert("posts", data).await {
            Ok(_) => println!("✓ 插入文章成功"),
            Err(e) => println!("✗ 插入文章失败: {}", e),
        }
    }

    println!();

    // 查询数据
    println!("3. 查询数据...");

    let query = QueryBuilder::select("users").build();
    match engine.query(query).await {
        Ok(result) => {
            println!("用户表数据:");
            print_table(&result.rows);
        }
        Err(e) => println!("✗ 查询用户失败: {}", e),
    }

    println!();

    let query = QueryBuilder::select("posts").build();
    match engine.query(query).await {
        Ok(result) => {
            println!("文章表数据:");
            print_table(&result.rows);
        }
        Err(e) => println!("✗ 查询文章失败: {}", e),
    }

    println!();

    // 条件查询
    println!("4. 条件查询...");
    let query = QueryBuilder::select("users")
        .where_condition("age", ComparisonOperator::GreaterThan, Value::Integer(30))
        .build();

    match engine.query(query).await {
        Ok(result) => {
            println!("年龄大于30的用户:");
            print_table(&result.rows);
        }
        Err(e) => println!("✗ 条件查询失败: {}", e),
    }

    println!();

    // 统计信息
    println!("5. 数据库统计信息...");
    show_stats(&engine).await;

    println!();
    println!("示例运行完成！");
    println!("你可以继续在交互模式中操作数据库，或输入 'exit' 退出。");
}