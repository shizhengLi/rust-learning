# 第十天：真实世界项目——从零构建完整的Rust应用

## 项目概述：构建一个任务管理系统

今天我们将构建一个完整的任务管理系统，它包含：
- REST API服务器
- 数据库持久化
- 异步任务处理
- 认证和授权
- 日志和监控
- 测试和部署

## 项目结构设计

```
task-manager/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── models/
│   │   ├── mod.rs
│   │   ├── task.rs
│   │   └── user.rs
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── tasks.rs
│   │   └── users.rs
│   ├── services/
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   └── email.rs
│   ├── database/
│   │   ├── mod.rs
│   │   └── connection.rs
│   └── utils/
│       ├── mod.rs
│       ├── logging.rs
│       └── errors.rs
├── tests/
│   └── integration.rs
├── migrations/
└── docker-compose.yml
```

## 项目配置

### Cargo.toml

```toml
[package]
name = "task-manager"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
warp = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.6", features = ["postgres", "runtime-tokio-rustls", "chrono", "uuid"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
bcrypt = "0.13"
jsonwebtoken = "8.0"
reqwest = { version = "0.11", features = ["json"] }
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
config = "0.13"
dotenv = "0.15"
validator = { version = "0.14", features = ["derive"] }
mockall = "0.11"
[dev-dependencies]
tokio-test = "0.4"
```

### 数据库模型

```rust
// src/models/mod.rs
pub mod task;
pub mod user;

pub use task::Task;
pub use user::User;

// src/models/task.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub assignee_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "task_status")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "task_priority")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
}

impl Task {
    pub fn new(
        title: String,
        description: Option<String>,
        priority: TaskPriority,
        assignee_id: Option<Uuid>,
        due_date: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            description,
            status: TaskStatus::Todo,
            priority,
            assignee_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            due_date,
        }
    }
}

// src/models/user.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role")]
pub enum UserRole {
    User,
    Admin,
}

impl User {
    pub fn new(username: String, email: String, password_hash: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash,
            role: UserRole::User,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_active: true,
        }
    }
}
```

## 数据库连接和迁移

```rust
// src/database/connection.rs
use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;

pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:password@localhost/taskmanager".to_string());

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;

        Ok(Database { pool })
    }

    pub async fn run_migrations(&self) -> Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }
}
```

## 错误处理

```rust
// src/utils/errors.rs
use thiserror::Error;
use warp::http::StatusCode;
use warp::reject::Reject;
use warp::Rejection;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    #[error("认证错误: {0}")]
    Auth(String),

    #[error("验证错误: {0}")]
    Validation(String),

    #[error("任务未找到")]
    TaskNotFound,

    #[error("用户未找到")]
    UserNotFound,

    #[error("JWT错误: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl Reject for ApiError {}

pub fn custom_reject(err: ApiError) -> Rejection {
    warp::reject::custom(err)
}

pub async fn handle_rejection(err: Rejection) -> Result<impl warp::Reply, Rejection> {
    if let Some(api_err) = err.find::<ApiError>() {
        let (code, message) = match api_err {
            ApiError::TaskNotFound => (StatusCode::NOT_FOUND, "任务未找到"),
            ApiError::UserNotFound => (StatusCode::NOT_FOUND, "用户未找到"),
            ApiError::Auth(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "内部服务器错误"),
        };

        let json = warp::reply::json(&serde_json::json!({
            "error": message,
            "code": code.as_u16(),
        }));

        Ok(warp::reply::with_status(json, code))
    } else {
        Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "error": "路由未找到",
                "code": 404,
            })),
            StatusCode::NOT_FOUND,
        ))
    }
}
```

## 认证服务

```rust
// src/services/auth.rs
use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // 用户ID
    pub username: String,
    pub role: String,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: String,
}

pub struct AuthService {
    secret: String,
}

impl AuthService {
    pub fn new(secret: String) -> Self {
        AuthService { secret }
    }

    pub fn hash_password(&self, password: &str) -> Result<String> {
        Ok(hash(password, DEFAULT_COST)?)
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        Ok(verify(password, hash)?)
    }

    pub fn generate_token(&self, user: &User) -> Result<String> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .expect("有效时间")
            .timestamp();

        let claims = Claims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            role: format!("{:?}", user.role),
            exp: expiration as usize,
        };

        Ok(encode(
            &Header::default(),
            &claims,
            &self.secret.as_ref(),
        )?)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<Claims>(token, self.secret.as_ref(), &validation)?;
        Ok(token_data.claims)
    }

    pub fn extract_token_from_header(header: &str) -> Result<&str> {
        if header.starts_with("Bearer ") {
            Ok(&header[7..])
        } else {
            Err(anyhow::anyhow!("无效的认证头"))
        }
    }
}
```

## 任务处理器

```rust
// src/handlers/tasks.rs
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::{Filter, Rejection, Reply};

use crate::database::Database;
use crate::models::{Task, TaskPriority, TaskStatus};
use crate::services::auth::Claims;
use crate::utils::errors::{ApiError, custom_reject};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub priority: TaskPriority,
    pub assignee_id: Option<Uuid>,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub assignee_id: Option<Uuid>,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
}

pub fn task_routes(
    db: Database,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    tasks_list(db.clone())
        .or(tasks_create(db.clone()))
        .or(tasks_get(db.clone()))
        .or(tasks_update(db.clone()))
        .or(tasks_delete(db))
}

// 获取所有任务
fn tasks_list(
    db: Database,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("tasks")
        .and(warp::get())
        .and(warp::path::end())
        .and(with_db(db))
        .and_then(|db: Database| async move {
            match list_tasks(&db).await {
                Ok(tasks) => Ok(warp::reply::json(&tasks)),
                Err(e) => Err(custom_reject(e)),
            }
        })
}

// 创建任务
fn tasks_create(
    db: Database,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("tasks")
        .and(warp::post())
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_db(db))
        .and(extract_claims())
        .and_then(|request: CreateTaskRequest, db: Database, claims: Claims| async move {
            match create_task(&db, request, claims).await {
                Ok(task) => Ok(warp::reply::with_status(
                    warp::reply::json(&task),
                    warp::http::StatusCode::CREATED,
                )),
                Err(e) => Err(custom_reject(e)),
            }
        })
}

// 获取单个任务
fn tasks_get(
    db: Database,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("tasks" / Uuid)
        .and(warp::get())
        .and(warp::path::end())
        .and(with_db(db))
        .and_then(|task_id: Uuid, db: Database| async move {
            match get_task(&db, task_id).await {
                Ok(task) => Ok(warp::reply::json(&task)),
                Err(e) => Err(custom_reject(e)),
            }
        })
}

// 更新任务
fn tasks_update(
    db: Database,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("tasks" / Uuid)
        .and(warp::put())
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_db(db))
        .and(extract_claims())
        .and_then(|task_id: Uuid, request: UpdateTaskRequest, db: Database, claims: Claims| async move {
            match update_task(&db, task_id, request, claims).await {
                Ok(task) => Ok(warp::reply::json(&task)),
                Err(e) => Err(custom_reject(e)),
            }
        })
}

// 删除任务
fn tasks_delete(
    db: Database,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("tasks" / Uuid)
        .and(warp::delete())
        .and(warp::path::end())
        .and(with_db(db))
        .and(extract_claims())
        .and_then(|task_id: Uuid, db: Database, claims: Claims| async move {
            match delete_task(&db, task_id, claims).await {
                Ok(_) => Ok(warp::reply::with_status(
                    warp::reply(),
                    warp::http::StatusCode::NO_CONTENT,
                )),
                Err(e) => Err(custom_reject(e)),
            }
        })
}

// 数据库操作函数
async fn list_tasks(db: &Database) -> Result<Vec<Task>, ApiError> {
    let tasks = sqlx::query_as::<_, Task>(
        "SELECT * FROM tasks ORDER BY created_at DESC",
    )
    .fetch_all(&db.pool)
    .await?;

    Ok(tasks)
}

async fn create_task(
    db: &Database,
    request: CreateTaskRequest,
    claims: Claims,
) -> Result<Task, ApiError> {
    let mut task = Task::new(
        request.title,
        request.description,
        request.priority,
        request.assignee_id,
        request.due_date,
    );

    let result = sqlx::query_as::<_, Task>(
        "INSERT INTO tasks (id, title, description, status, priority, assignee_id, created_at, updated_at, due_date)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         RETURNING *",
    )
    .bind(task.id)
    .bind(&task.title)
    .bind(&task.description)
    .bind(&task.status)
    .bind(&task.priority)
    .bind(&task.assignee_id)
    .bind(task.created_at)
    .bind(task.updated_at)
    .bind(&task.due_date)
    .fetch_one(&db.pool)
    .await?;

    Ok(result)
}

async fn get_task(db: &Database, task_id: Uuid) -> Result<Task, ApiError> {
    let task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1")
        .bind(task_id)
        .fetch_optional(&db.pool)
        .await?;

    match task {
        Some(task) => Ok(task),
        None => Err(ApiError::TaskNotFound),
    }
}

async fn update_task(
    db: &Database,
    task_id: Uuid,
    request: UpdateTaskRequest,
    claims: Claims,
) -> Result<Task, ApiError> {
    let mut task = get_task(db, task_id).await?;

    // 更新字段
    if let Some(title) = request.title {
        task.title = title;
    }
    if let Some(description) = request.description {
        task.description = Some(description);
    }
    if let Some(status) = request.status {
        task.status = status;
    }
    if let Some(priority) = request.priority {
        task.priority = priority;
    }
    if let Some(assignee_id) = request.assignee_id {
        task.assignee_id = Some(assignee_id);
    }
    if let Some(due_date) = request.due_date {
        task.due_date = Some(due_date);
    }

    task.updated_at = chrono::Utc::now();

    let result = sqlx::query_as::<_, Task>(
        "UPDATE tasks SET title = $1, description = $2, status = $3, priority = $4,
         assignee_id = $5, updated_at = $6, due_date = $7
         WHERE id = $8 RETURNING *",
    )
    .bind(&task.title)
    .bind(&task.description)
    .bind(&task.status)
    .bind(&task.priority)
    .bind(&task.assignee_id)
    .bind(task.updated_at)
    .bind(&task.due_date)
    .bind(task.id)
    .fetch_one(&db.pool)
    .await?;

    Ok(result)
}

async fn delete_task(db: &Database, task_id: Uuid, claims: Claims) -> Result<(), ApiError> {
    let result = sqlx::query("DELETE FROM tasks WHERE id = $1")
        .bind(task_id)
        .execute(&db.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::TaskNotFound);
    }

    Ok(())
}

// 辅助函数
fn with_db(db: Database) -> impl Filter<Extract = (Database,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn extract_claims() -> impl Filter<Extract = (Claims,), Error = Rejection> + Clone {
    warp::header::<String>("authorization")
        .and_then(|auth_header: String| async move {
            let auth_service = crate::services::auth::AuthService::new(
                std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string()),
            );

            match auth_service.extract_token_from_header(&auth_header) {
                Ok(token) => {
                    match auth_service.verify_token(token) {
                        Ok(claims) => Ok(claims),
                        Err(e) => Err(custom_reject(ApiError::Auth(e.to_string()))),
                    }
                }
                Err(e) => Err(custom_reject(ApiError::Auth(e.to_string()))),
            }
        })
}
```

## 邮件服务（异步任务处理）

```rust
// src/services/email.rs
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Debug, Serialize, Deserialize)]
pub struct Email {
    pub to: String,
    pub subject: String,
    pub body: String,
}

pub struct EmailService {
    sender: mpsc::UnboundedSender<Email>,
}

impl EmailService {
    pub fn new() -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel::<Email>();

        // 启动后台任务处理邮件
        tokio::spawn(async move {
            while let Some(email) = receiver.recv().await {
                if let Err(e) = Self::send_email(email).await {
                    eprintln!("发送邮件失败: {}", e);
                }
            }
        });

        EmailService { sender }
    }

    pub async fn send_email(&self, email: Email) -> Result<()> {
        // 这里可以集成真实的邮件服务
        println!("发送邮件到 {}: {}", email.to, email.subject);

        // 模拟发送延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Ok(())
    }

    pub fn send_async(&self, email: Email) -> Result<()> {
        self.sender.send(email)?;
        Ok(())
    }
}

// 任务通知服务
pub struct NotificationService {
    email_service: EmailService,
}

impl NotificationService {
    pub fn new(email_service: EmailService) -> Self {
        NotificationService { email_service }
    }

    pub fn send_task_assigned_notification(&self, email: &str, task_title: &str) -> Result<()> {
        let email = Email {
            to: email.to_string(),
            subject: format!("新任务分配: {}", task_title),
            body: format!("你被分配了一个新任务: {}", task_title),
        };

        self.email_service.send_async(email)?;
        Ok(())
    }

    pub fn send_task_due_notification(&self, email: &str, task_title: &str, due_date: &str) -> Result<()> {
        let email = Email {
            to: email.to_string(),
            subject: format!("任务即将到期: {}", task_title),
            body: format!("任务 '{}' 将在 {} 到期", task_title, due_date),
        };

        self.email_service.send_async(email)?;
        Ok(())
    }
}
```

## 主应用程序

```rust
// src/main.rs
use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber;
use warp::Filter;

use task_manager::database::Database;
use task_manager::handlers::{task_routes, user_routes};
use task_manager::services::auth::AuthService;
use task_manager::services::email::{EmailService, NotificationService};
use task_manager::utils::errors::handle_rejection;
use task_manager::utils::logging;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    logging::init();

    info!("启动任务管理系统");

    // 加载配置
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost/taskmanager".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-secret-key".to_string());

    // 初始化数据库
    let db = Database::new().await?;
    db.run_migrations().await?;
    info!("数据库连接成功");

    // 初始化服务
    let auth_service = AuthService::new(jwt_secret);
    let email_service = EmailService::new();
    let notification_service = NotificationService::new(email_service);

    // CORS配置
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["authorization", "content-type"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

    // 路由配置
    let routes = task_routes(db.clone())
        .or(user_routes(db.clone()))
        .with(cors)
        .with(handle_rejection());

    // 启动服务器
    info!("服务器启动在 http://localhost:3030");
    warp::serve(routes)
        .run(([0, 0, 0, 0], 3030))
        .await;

    Ok(())
}
```

## 测试

```rust
// tests/integration.rs
use task_manager::database::Database;
use task_manager::models::{Task, TaskPriority, TaskStatus};
use warp::http::StatusCode;
use warp::test::request;

#[tokio::test]
async fn test_create_task() {
    // 设置测试数据库
    let db = setup_test_db().await;

    // 测试创建任务
    let response = request()
        .method("POST")
        .path("/tasks")
        .json(&serde_json::json!({
            "title": "测试任务",
            "description": "这是一个测试任务",
            "priority": "Medium"
        }))
        .reply(&task_routes(db))
        .await;

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_get_tasks() {
    let db = setup_test_db().await;

    // 先创建一个任务
    request()
        .method("POST")
        .path("/tasks")
        .json(&serde_json::json!({
            "title": "测试任务",
            "priority": "High"
        }))
        .reply(&task_routes(db.clone()))
        .await;

    // 获取任务列表
    let response = request()
        .method("GET")
        .path("/tasks")
        .reply(&task_routes(db))
        .await;

    assert_eq!(response.status(), StatusCode::OK);
}

async fn setup_test_db() -> Database {
    let db = Database::new().await.unwrap();
    db.run_migrations().await.unwrap();
    db
}
```

## Docker部署

### docker-compose.yml

```yaml
version: '3.8'

services:
  app:
    build: .
    ports:
      - "3030:3030"
    environment:
      - DATABASE_URL=postgres://postgres:password@db:5432/taskmanager
      - JWT_SECRET=your-production-secret-key
      - RUST_LOG=info
    depends_on:
      - db
    volumes:
      - ./logs:/app/logs

  db:
    image: postgres:13
    environment:
      - POSTGRES_DB=taskmanager
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  redis:
    image: redis:6
    ports:
      - "6379:6379"

volumes:
  postgres_data:
```

### Dockerfile

```dockerfile
FROM rust:1.70 as builder

WORKDIR /app

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./

# 构建依赖
RUN cargo build --release

# 复制源代码
COPY src ./src
COPY migrations ./migrations

# 构建应用
RUN cargo build --release

# 运行时镜像
FROM debian:bullseye-slim

WORKDIR /app

# 复制构建的二进制文件
COPY --from=builder /app/target/release/task-manager .

# 复制配置文件
COPY --from=builder /app/migrations ./migrations

# 安装必要的运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# 创建非root用户
RUN useradd -m -u 1000 appuser
USER appuser

# 暴露端口
EXPOSE 3030

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:3030/health || exit 1

CMD ["./task-manager"]
```

## 性能优化

### 1. 数据库优化

```rust
// 使用连接池
use sqlx::postgres::PgPoolOptions;

pub async fn create_optimized_pool() -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .idle_timeout(std::time::Duration::from_secs(600))
        .max_lifetime(std::time::Duration::from_secs(1800))
        .connect(&database_url)
        .await
}
```

### 2. 缓存策略

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct Cache<T> {
    data: Arc<RwLock<HashMap<Uuid, T>>>,
    ttl: std::time::Duration,
}

impl<T: Clone> Cache<T> {
    pub fn new(ttl: std::time::Duration) -> Self {
        Cache {
            data: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    pub async fn get(&self, key: &Uuid) -> Option<T> {
        let data = self.data.read().await;
        data.get(key).cloned()
    }

    pub async fn put(&self, key: Uuid, value: T) {
        let mut data = self.data.write().await;
        data.insert(key, value);

        // 设置TTL
        let data = self.data.clone();
        tokio::spawn(async move {
            tokio::time::sleep(self.ttl).await;
            let mut data = data.write().await;
            data.remove(&key);
        });
    }
}
```

### 3. 监控和指标

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct Metrics {
    pub requests_total: Arc<AtomicU64>,
    pub requests_duration_ms: Arc<AtomicU64>,
    pub active_connections: Arc<AtomicU64>,
}

impl Metrics {
    pub fn new() -> Self {
        Metrics {
            requests_total: Arc::new(AtomicU64::new(0)),
            requests_duration_ms: Arc::new(AtomicU64::new(0)),
            active_connections: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn increment_requests(&self) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_duration(&self, duration_ms: u64) {
        self.requests_duration_ms.fetch_add(duration_ms, Ordering::Relaxed);
    }

    pub fn get_stats(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        stats.insert("requests_total".to_string(), self.requests_total.load(Ordering::Relaxed));
        stats.insert("requests_duration_ms".to_string(), self.requests_duration_ms.load(Ordering::Relaxed));
        stats.insert("active_connections".to_string(), self.active_connections.load(Ordering::Relaxed));
        stats
    }
}
```

## 最佳实践总结

### 1. 错误处理策略

```rust
// 使用anyhow进行错误传播
async fn process_request() -> Result<Response> {
    let data = fetch_data().await.context("获取数据失败")?;
    let processed = process_data(&data).await.context("处理数据失败")?;
    Ok(create_response(processed))
}

// 使用thiserror定义错误类型
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("网络错误: {0}")]
    Network(String),
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
}
```

### 2. 配置管理

```rust
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub jwt: JwtSettings,
    pub server: ServerSettings,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize)]
pub struct JwtSettings {
    pub secret: String,
    pub expiration_hours: i64,
}

#[derive(Debug, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut settings = Config::builder();

        // 从配置文件读取
        settings = settings.add_source(File::with_name("config/default"));

        // 从环境变量覆盖
        settings = settings.add_source(Environment::with_prefix("APP"));

        settings.build()?.try_deserialize()
    }
}
```

### 3. 日志记录

```rust
use tracing::{debug, error, info, span, Level, Span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "task_manager=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

pub async fn process_task(task_id: Uuid) -> Result<()> {
    let span = span!(Level::INFO, "process_task", task_id = %task_id);
    let _enter = span.enter();

    info!("开始处理任务");

    match do_work(task_id).await {
        Ok(_) => {
            info!("任务处理成功");
            Ok(())
        }
        Err(e) => {
            error!(error = %e, "任务处理失败");
            Err(e)
        }
    }
}
```

### 4. 测试策略

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        Database {}
        impl DatabaseTrait for Database {
            async fn get_task(&self, id: Uuid) -> Result<Task>;
            async fn save_task(&self, task: &Task) -> Result<()>;
        }
    }

    #[tokio::test]
    async fn test_task_processing() {
        let mut mock_db = MockDatabase::new();

        // 设置mock行为
        mock_db.expect_get_task()
            .returning(|_| Ok(Task::new("测试".to_string(), None, TaskPriority::Medium, None, None)));

        mock_db.expect_save_task()
            .returning(|_| Ok(()));

        // 执行测试
        let result = process_task(&mock_db, Uuid::new_v4()).await;
        assert!(result.is_ok());
    }
}
```

## 总结

今天我们构建了一个完整的Rust应用，涵盖了：

1. **项目结构**：模块化设计
2. **数据库**：ORM和迁移
3. **API设计**：RESTful接口
4. **认证**：JWT集成
5. **异步处理**：邮件通知
6. **错误处理**：统一错误类型
7. **测试**：单元测试和集成测试
8. **部署**：Docker容器化
9. **监控**：性能指标
10. **最佳实践**：配置、日志、测试

**关键点：**
- 使用现代Rust生态系统构建生产级应用
- 异步编程提供高性能
- 类型安全保证代码质量
- 模块化设计提高可维护性

## 进阶学习路径

1. **深入异步编程**：tokio内部原理
2. **性能优化**：零拷贝、内存池
3. **分布式系统**：gRPC、消息队列
4. **机器学习**：tch、candle等库
5. **嵌入式开发**：no_std环境

**恭喜！你已经完成了Rust从入门到实战的学习旅程！**

---

*这个项目展示了Rust在现代Web开发中的强大能力。结合你已有的Python/C++背景，你可以选择在Rust中构建高性能系统，或者在与现有系统的集成中发挥Rust的优势。*