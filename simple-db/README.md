# Simple DB - 一个简单的内存数据库

Simple DB 是一个用 Rust 实现的轻量级内存数据库，提供了基本的 SQL 功能和持久化支持。这个项目旨在演示数据库系统的核心概念，包括存储引擎、查询引擎、事务处理等。

## 特性

- 🗄️ **完整的数据类型支持** - Integer, Text, Boolean, Float, Date, Time, DateTime, JSON, Binary
- 🔍 **强大的查询功能** - 条件查询、排序、分页、聚合函数
- 💾 **持久化支持** - 事务日志、快照、备份恢复
- 🔄 **事务处理** - ACID 特性支持
- 🚀 **高性能** - 批量操作、索引优化
- 🛡️ **数据完整性** - 主键、唯一约束、非空约束
- 💻 **交互式CLI** - 友好的命令行界面
- 🧪 **完整的测试** - 单元测试和集成测试

## 快速开始

### 安装

确保你已经安装了 Rust 工具链：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 克隆项目

```bash
git clone <repository-url>
cd simple-db
```

### 构建

```bash
cargo build --release
```

### 运行

```bash
# 启动交互式Shell
cargo run -- shell

# 运行示例
cargo run -- example

# 执行SQL文件
cargo run -- execute -f examples/your_script.sql
```

## 使用示例

### 基本使用

```rust
use simple_db::engine::DatabaseEngine;
use simple_db::query::{QueryBuilder, ComparisonOperator};
use simple_db::types::{Value, DataType, Schema, ColumnDefinition};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建数据库引擎
    let mut engine = DatabaseEngine::new();

    // 创建表
    let schema = Schema::new(vec![
        ColumnDefinition::new("id", DataType::Integer, true),
        ColumnDefinition::new("name", DataType::Text, false),
        ColumnDefinition::new("age", DataType::Integer, false),
    ]);

    engine.create_table("users", schema).await?;

    // 插入数据
    let mut data = std::collections::HashMap::new();
    data.insert("id".to_string(), Value::Integer(1));
    data.insert("name".to_string(), Value::Text("Alice".to_string()));
    data.insert("age".to_string(), Value::Integer(25));

    engine.insert("users", data).await?;

    // 查询数据
    let query = QueryBuilder::select("users")
        .where_condition("age", ComparisonOperator::GreaterThan, Value::Integer(20))
        .build();

    let result = engine.query(query).await?;
    for row in &result.rows {
        println!("User: {}", row.get("name").unwrap());
    }

    Ok(())
}
```

### 交互式Shell

启动交互式Shell后，你可以使用以下命令：

```sql
-- 创建表
CREATE TABLE users

-- 插入数据
INSERT INTO users

-- 查询数据
SELECT * FROM users

-- 更新数据
UPDATE users SET ...

-- 删除数据
DELETE FROM users

-- 查看表结构
DESCRIBE users

-- 统计行数
COUNT FROM users

-- 列出所有表
tables

-- 保存数据库
save

-- 加载数据库
load

-- 查看统计信息
stats
```

## 项目结构

```
simple-db/
├── src/
│   ├── lib.rs              # 主库文件
│   ├── main.rs             # CLI入口
│   ├── error.rs            # 错误处理
│   ├── types.rs            # 数据类型定义
│   ├── storage.rs          # 存储引擎
│   ├── query.rs            # 查询引擎
│   └── engine.rs           # 数据库引擎
├── examples/
│   ├── basic_usage.rs      # 基本使用示例
│   └── advanced_features.rs # 高级特性示例
├── tests/
│   └── integration_test.rs  # 集成测试
├── data/                   # 数据文件目录
├── Cargo.toml             # 项目配置
└── README.md              # 项目说明
```

## API 文档

### 核心类型

#### DatabaseEngine
主要的数据库引擎，提供所有数据库操作：

```rust
use simple_db::engine::DatabaseEngine;

let mut engine = DatabaseEngine::new();

// 创建表
engine.create_table("table_name", schema).await?;

// 插入数据
let id = engine.insert("table_name", data).await?;

// 查询数据
let result = engine.query(query).await?;

// 更新数据
let affected = engine.update("table_name", conditions, updates).await?;

// 删除数据
let affected = engine.delete("table_name", conditions).await?;
```

#### QueryBuilder
构建查询对象的辅助类：

```rust
use simple_db::query::{QueryBuilder, ComparisonOperator};

// 构建查询
let query = QueryBuilder::select("users")
    .where_condition("age", ComparisonOperator::GreaterThan, Value::Integer(25))
    .order_by("name", true)
    .limit(10)
    .build();
```

#### 数据类型

```rust
use simple_db::types::{Value, DataType, Schema, ColumnDefinition};

// 值类型
let int_val = Value::Integer(42);
let text_val = Value::Text("Hello".to_string());
let bool_val = Value::Boolean(true);

// 数据类型
let data_type = DataType::Integer;

// 列定义
let column = ColumnDefinition::new("id", DataType::Integer, true)
    .unique(true)
    .nullable(false);

// 表结构
let schema = Schema::new(vec![column]);
```

### 支持的操作

#### 查询操作
- `SELECT` - 基本查询
- `WHERE` - 条件过滤
- `ORDER BY` - 排序
- `LIMIT/OFFSET` - 分页
- `COUNT` - 计数

#### 条件操作符
- `Equal` (=)
- `NotEqual` (!=)
- `GreaterThan` (>)
- `GreaterThanOrEqual` (>=)
- `LessThan` (<)
- `LessThanOrEqual` (<=)
- `Like` - 模糊匹配
- `In` - IN操作
- `IsNull` / `IsNotNull` - NULL检查

#### 约束支持
- 主键约束
- 唯一约束
- 非空约束
- 默认值

## 高级特性

### 事务处理

```rust
use simple_db::engine::DatabaseEngine;

let result = engine.transaction(|tx| {
    // 创建表
    tx.create_table("trans_test", schema)?;

    // 插入数据
    tx.insert("trans_test", data1)?;
    tx.insert("trans_test", data2)?;

    // 更新数据
    tx.update("trans_test", id, updates)?;

    // 删除数据
    tx.delete("trans_test", id)?;

    Ok(())
}).await;
```

### 批量操作

```rust
// 批量插入
let mut batch_data = Vec::new();
for i in 0..1000 {
    let mut data = HashMap::new();
    data.insert("id".to_string(), Value::Integer(i));
    // 添加更多字段...
    batch_data.push(data);
}

let ids = engine.batch_insert("table_name", batch_data).await?;
```

### 备份和恢复

```rust
// 备份
engine.backup("/path/to/backup").await?;

// 恢复
engine.restore("/path/to/backup").await?;
```

## 存储架构

### 内存存储
- 表结构存储在内存中
- 支持快速查询和更新
- 数据可持久化到磁盘

### 持久化机制
- **事务日志** - 记录所有操作，支持故障恢复
- **快照机制** - 定期保存完整状态
- **WAL** - Write-Ahead Logging 保证数据一致性

### 文件格式
- `data/snapshot.json` - 数据库快照
- `data/transaction.log` - 事务日志

## 性能特性

### 内存优化
- 使用 Rust 的所有权系统避免内存泄漏
- 智能指针管理数据生命周期
- 高效的数据结构

### 查询优化
- 索引支持（开发中）
- 查询计划优化
- 批量操作优化

### 并发支持
- 基于 tokio 的异步IO
- 读写锁保证并发安全
- 事务隔离级别

## 开发和测试

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行集成测试
cargo test --test integration_test

# 运行示例
cargo run --example basic_usage
cargo run --example advanced_features
```

### 代码质量

```bash
# 格式化代码
cargo fmt

# 检查代码
cargo clippy

# 生成文档
cargo doc
```

## 路线图

### 即将发布的功能
- [ ] 索引支持
- [ ] 连接查询 (JOIN)
- [ ] 分组查询 (GROUP BY)
- [ ] 聚合函数 (SUM, AVG, MAX, MIN)
- [ ] 触发器
- [ ] 视图
- [ ] 用户权限管理
- [ ] 网络协议支持
- [ ] SQL解析器

### 长期规划
- [ ] 分布式支持
- [ ] 查询优化器
- [ ] 存储过程
- [ ] 全文搜索
- [ ] 时间序列支持
- [ ] 地理空间数据

## 贡献指南

欢迎贡献代码！请遵循以下步骤：

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

### 开发环境设置

```bash
# 克隆项目
git clone <repository-url>
cd simple-db

# 安装开发依赖
cargo install cargo-watch cargo-tarpaulin

# 运行开发服务器
cargo watch -x run

# 运行测试并生成覆盖率报告
cargo tarpaulin --out Html
```

## 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 致谢

- Rust 语言和社区
- SQLite 的设计理念
- PostgreSQL 的高级特性
- 各种优秀的开源数据库项目

## 支持

如果你在使用过程中遇到问题，请：

1. 查看 [文档](https://docs.rs/simple-db)
2. 搜索现有的 [Issues](https://github.com/your-username/simple-db/issues)
3. 创建新的 Issue 描述问题

---

**Simple DB** - 让数据库学习变得简单有趣！ 🚀