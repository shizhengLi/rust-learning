# 第九天：高级主题——宏、异步编程和生态系统

## 宏（Macros）：元编程的威力

Rust的宏系统允许你在编译时生成代码，提供强大的元编程能力。

### 从C++的角度理解

**C++的宏：**
```cpp
#define LOG(x) std::cout << x << std::endl

#define MAX(a, b) ((a) > (b) ? (a) : (b))

// 使用
LOG("Hello, C++");
int result = MAX(5, 3);
```

**Rust的宏：**
```rust
macro_rules! log {
    ($($arg:tt)*) => {
        println!($($arg)*);
    };
}

macro_rules! max {
    ($a:expr, $b:expr) => {
        if $a > $b { $a } else { $b }
    };
}

// 使用
log!("Hello, Rust");
let result = max!(5, 3);
```

### 从Python的角度理解

**Python的装饰器和元编程：**
```python
def log_decorator(func):
    def wrapper(*args, **kwargs):
        print(f"调用函数: {func.__name__}")
        return func(*args, **kwargs)
    return wrapper

@log_decorator
def my_function():
    print("函数执行中")

# 使用
my_function()
```

### 声明式宏（Declarative Macros）

```rust
// 基本语法
macro_rules! create_function {
    ($func_name:ident) => {
        fn $func_name() {
            println!("函数 {} 被调用", stringify!($func_name));
        }
    };
}

// 创建函数
create_function!(hello);
create_function!(goodbye);

fn main() {
    hello();    // 输出: 函数 hello 被调用
    goodbye();  // 输出: 函数 goodbye 被调用
}
```

### 复杂的宏示例

```rust
macro_rules! vec {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $( temp_vec.push($x); )*
            temp_vec
        }
    };
}

// 实现自己的assert宏
macro_rules! my_assert {
    ($condition:expr) => {
        if !$condition {
            panic!("断言失败: {}", stringify!($condition));
        }
    };
    ($condition:expr, $($arg:tt)*) => {
        if !$condition {
            panic!("断言失败: {}: {}", stringify!($condition), format!($($arg)*));
        }
    };
}

fn use_macros() {
    let v = vec![1, 2, 3];
    println!("向量: {:?}", v);

    my_assert!(2 + 2 == 4);
    my_assert!(1 + 1 == 3, "数学出错了！");
}
```

### 过程式宏（Procedural Macros）

```rust
// 首先在Cargo.toml中添加依赖
// [lib]
// proc-macro = true

// [dependencies]
// quote = "1.0"
// syn = "1.0"

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(HelloMacro)]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl HelloMacro for #name {
            fn hello_macro() {
                println!("你好，我是 {}！", stringify!(#name));
            }
        }
    };

    expanded.into()
}

trait HelloMacro {
    fn hello_macro();
}

#[derive(HelloMacro)]
struct Pancakes;

fn use_procedural_macro() {
    Pancakes::hello_macro(); // 输出: 你好，我是 Pancakes！
}
```

## 异步编程（Async/Await）

异步编程允许你编写高效的并发代码，而不需要传统的线程。

### 从Python的角度理解

**Python的async/await：**
```python
import asyncio

async def fetch_data(url):
    print(f"开始获取 {url}")
    await asyncio.sleep(1)
    print(f"完成获取 {url}")
    return f"数据来自 {url}"

async def main():
    task1 = fetch_data("https://example.com")
    task2 = fetch_data("https://example.org")
    results = await asyncio.gather(task1, task2)
    print(results)

asyncio.run(main())
```

**Rust的async/await：**
```rust
use std::time::Duration;
use tokio::time::sleep;

async fn fetch_data(url: &str) -> String {
    println!("开始获取 {}", url);
    sleep(Duration::from_secs(1)).await;
    println!("完成获取 {}", url);
    format!("数据来自 {}", url)
}

#[tokio::main]
async fn main() {
    let task1 = fetch_data("https://example.com");
    let task2 = fetch_data("https://example.org");

    let (result1, result2) = tokio::join!(task1, task2);
    println!("结果1: {}, 结果2: {}", result1, result2);
}
```

### Future和Executor

```rust
use std::future::Future;
use std::pin::Pin;

// 简单的Future实现
struct SimpleFuture {
    state: u32,
}

impl Future for SimpleFuture {
    type Output = u32;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut std::task::Context) -> std::task::Poll<Self::Output> {
        if self.state < 3 {
            println!("Polling: state = {}", self.state);
            self.state += 1;
            std::task::Poll::Pending
        } else {
            std::task::Poll::Ready(self.state)
        }
    }
}

fn use_simple_future() {
    let future = SimpleFuture { state: 0 };
    // 需要一个executor来运行这个future
    println!("简单Future示例（概念性）");
}
```

### 异步文件操作

```rust
use tokio::fs;
use tokio::io;

async fn async_file_operations() -> io::Result<()> {
    // 异步写入文件
    let content = "Hello, async world!";
    fs::write("hello.txt", content).await?;

    // 异步读取文件
    let read_content = fs::read_to_string("hello.txt").await?;
    println!("读取的内容: {}", read_content);

    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    async_file_operations().await
}
```

### 异步迭代器

```rust
use tokio::stream::{self, StreamExt};

async fn async_streams() {
    let mut stream = stream::iter(1..=10);

    while let Some(item) = stream.next().await {
        println!("流元素: {}", item);
    }
}

// 自定义异步迭代器
struct Counter {
    count: u32,
}

impl Counter {
    fn new() -> Self {
        Counter { count: 0 }
    }
}

impl futures::Stream for Counter {
    type Item = u32;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context,
    ) -> std::task::Poll<Option<Self::Item>> {
        if self.count < 5 {
            let val = self.count;
            self.count += 1;
            std::task::Poll::Ready(Some(val))
        } else {
            std::task::Poll::Ready(None)
        }
    }
}

async fn use_custom_stream() {
    let mut counter = Counter::new();
    while let Some(value) = counter.next().await {
        println!("计数器: {}", value);
    }
}
```

## Rust生态系统

### 包管理器Cargo

```rust
// Cargo.toml示例
[package]
name = "my_project"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
reqwest = { version = "0.11", features = ["json"] }
sqlx = { version = "0.6", features = ["postgres", "runtime-tokio-rustls"] }
anyhow = "1.0"
thiserror = "1.0"
```

### 常用库推荐

#### 1. 序列化/反序列化 - serde

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: u32,
    name: String,
    email: String,
}

fn use_serde() {
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    // 序列化为JSON
    let json = serde_json::to_string(&user).unwrap();
    println!("JSON: {}", json);

    // 从JSON反序列化
    let deserialized: User = serde_json::from_str(&json).unwrap();
    println!("反序列化: {:?}", deserialized);
}
```

#### 2. HTTP客户端 - reqwest

```rust
use reqwest;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Post {
    userId: u32,
    id: u32,
    title: String,
    body: String,
}

#[tokio::main]
async fn fetch_posts() -> Result<(), reqwest::Error> {
    let response = reqwest::get("https://jsonplaceholder.typicode.com/posts/1")
        .await?
        .json::<Post>()
        .await?;

    println!("文章: {:?}", response);
    Ok(())
}
```

#### 3. 数据库 - sqlx

```rust
use sqlx::postgres::PgPoolOptions;
use sqlx::Row;

#[derive(sqlx::FromRow)]
struct User {
    id: i32,
    name: String,
}

#[tokio::main]
async fn use_sqlx() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://user:password@localhost/database")
        .await?;

    // 查询
    let user = sqlx::query_as::<_, User>("SELECT id, name FROM users WHERE id = $1")
        .bind(1)
        .fetch_one(&pool)
        .await?;

    println!("用户: {}", user.name);

    // 插入
    let result = sqlx::query("INSERT INTO users (name) VALUES ($1)")
        .bind("Alice")
        .execute(&pool)
        .await?;

    println!("插入了 {} 行", result.rows_affected());

    Ok(())
}
```

#### 4. 错误处理 - anyhow和thiserror

```rust
use anyhow::{Context, Result};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("网络错误: {0}")]
    Network(String),

    #[error("解析错误")]
    Parse,
}

fn use_error_libraries() -> Result<()> {
    let content = std::fs::read_to_string("config.json")
        .context("读取配置文件失败")?;

    // 处理内容...
    Ok(())
}
```

### 测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    #[should_panic(expected = "断言失败")]
    fn test_panicking() {
        panic!("这是一个测试用的panic");
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}

// 集成测试
// tests/integration_test.rs
#[cfg(test)]
mod tests {
    #[test]
    fn test_integration() {
        // 集成测试代码
    }
}
```

### 基准测试

```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_vector_creation(b: &mut Bencher) {
        b.iter(|| {
            let v: Vec<i32> = (0..1000).collect();
            v
        });
    }

    #[bench]
    fn bench_hashmap_insertion(b: &mut Bencher) {
        let mut map = std::collections::HashMap::new();
        b.iter(|| {
            map.insert(0, 0);
        });
    }
}
```

## 实战例子：异步Web服务器

```rust
use warp::Filter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    id: u32,
    content: String,
    author: String,
}

type Messages = Arc<RwLock<HashMap<u32, Message>>>;

#[tokio::main]
async fn main() {
    let messages: Messages = Arc::new(RwLock::new(HashMap::new()));

    // 获取所有消息
    let messages_get = messages.clone();
    let get_messages = warp::get()
        .and(warp::path("messages"))
        .and(warp::path::end())
        .map(move || {
            let messages = messages_get.blocking_read();
            warp::reply::json(&*messages)
        });

    // 创建新消息
    let messages_post = messages.clone();
    let post_messages = warp::post()
        .and(warp::path("messages"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |mut message: Message| {
            let mut messages = messages_post.blocking_write();
            message.id = messages.len() as u32 + 1;
            messages.insert(message.id, message.clone());
            warp::reply::json(&message)
        });

    // 获取单个消息
    let messages_get_one = messages.clone();
    let get_message = warp::get()
        .and(warp::path("messages"))
        .and(warp::path::param::<u32>())
        .and(warp::path::end())
        .map(move |id: u32| {
            let messages = messages_get_one.blocking_read();
            match messages.get(&id) {
                Some(message) => warp::reply::json(message),
                None => warp::reply::with_status("消息未找到", warp::http::StatusCode::NOT_FOUND),
            }
        });

    let routes = get_messages
        .or(post_messages)
        .or(get_message);

    println!("服务器运行在 http://localhost:3030");
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
```

## 实战例子：使用宏实现DSL

```rust
macro_rules! css {
    (
        $(
            $property:ident : $value:expr
        ),* $(,)?
    ) => {
        {
            let mut styles = String::new();
            $(
                styles.push_str(&format!("{}: {}; ", stringify!($property), $value));
            )*
            styles
        }
    };
}

macro_rules! html {
    (
        $tag:ident
        $( [$($attr:ident = $val:expr),*] )?
        $( { $($children:tt)* } )?
    ) => {
        {
            let mut output = String::new();
            output.push_str(&format!("<{}", stringify!($tag)));

            $(
                $(
                    output.push_str(&format!(" {}=\"{}\"", stringify!($attr), $val));
                )*
            )?

            output.push('>');

            $(
                $(
                    output.push_str(&html!($children));
                )*
            )?

            output.push_str(&format!("</{}>", stringify!($tag)));
            output
        }
    };

    ( $text:expr ) => {
        $text.to_string()
    };
}

fn use_html_dsl() {
    let page = html! {
        html {
            head {
                title { "我的网页" }
            }
            body [style = css!(background_color: "white", color: "black")] {
                h1 { "欢迎来到我的网站" }
                p { "这是一个使用宏生成的HTML页面。" }
                div [class = "container"] {
                    p { "内容区域" }
                }
            }
        }
    };

    println!("{}", page);
}
```

## 总结

今天我们学习了：
- **宏系统**：声明式宏和过程式宏
- **异步编程**：async/await、Future、Stream
- **生态系统**：Cargo、常用库、测试、基准测试
- **实战项目**：Web服务器、DSL实现

**关键点：**
- 宏提供了强大的元编程能力
- 异步编程是高性能网络服务的核心
- Rust生态系统丰富且成熟
- 测试是Rust开发的重要组成部分

**明天预告：** 真实世界项目——从零构建一个完整的应用！

## 练习作业

1. 创建一个自定义的derive宏，自动为结构体实现验证逻辑
2. 实现一个异步的TCP聊天室服务器
3. 使用Rust生态系统构建一个完整的REST API服务

---

*如果你是C++开发者，重点关注Rust的元编程能力和现代异步编程模型；如果你是Python开发者，重点关注Rust如何在保持生产力的同时提供更高的性能和安全性。*