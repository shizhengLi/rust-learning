# 第五天：错误处理——Result和Option的优雅艺术

## 错误处理的重要性

错误处理是编程中不可避免的挑战。不同的语言有不同的错误处理方式，每种都有其优缺点。

### 从Python的角度理解

**Python的异常处理：**
```python
def divide(a: float, b: float) -> float:
    if b == 0:
        raise ValueError("除数不能为零")
    return a / b

# 使用try-except
try:
    result = divide(10, 0)
except ValueError as e:
    print(f"错误: {e}")
```

### 从C++的角度理解

**C++的异常处理：**
```cpp
#include <stdexcept>

double divide(double a, double b) {
    if (b == 0) {
        throw std::invalid_argument("除数不能为零");
    }
    return a / b;
}

// 使用try-catch
try {
    double result = divide(10, 0);
} catch (const std::invalid_argument& e) {
    std::cout << "错误: " << e.what() << std::endl;
}
```

## Option类型：处理可能为空的值

`Option<T>`是Rust中处理可能为空的值的枚举类型。

```rust
enum Option<T> {
    Some(T),  // 包含一个值
    None,     // 不包含值
}
```

### 基本用法

```rust
fn find_first_word(s: &str) -> Option<&str> {
    s.split(' ').next()
}

fn main() {
    let sentence = "Hello world";
    let empty = "";

    // 使用match
    match find_first_word(sentence) {
        Some(word) => println!("第一个单词: {}", word),
        None => println!("没有找到单词"),
    }

    match find_first_word(empty) {
        Some(word) => println!("第一个单词: {}", word),
        None => println!("没有找到单词"),
    }
}
```

### Option的常用方法

```rust
fn main() {
    let some_value = Some(5);
    let none_value: Option<i32> = None;

    // unwrap - 存在风险
    // let value = none_value.unwrap(); // 会panic！

    // expect - 带错误信息的unwrap
    // let value = none_value.expect("这里不应该为None"); // 会panic！

    // unwrap_or - 提供默认值
    let value = none_value.unwrap_or(0);
    println!("值: {}", value); // 输出: 0

    // unwrap_or_else - 使用函数提供默认值
    let value = none_value.unwrap_or_else(|| 0);
    println!("值: {}", value);

    // map - 转换Some内部的值
    let doubled = some_value.map(|x| x * 2);
    println!("双倍值: {:?}", doubled); // 输出: Some(10)

    // and_then - 链式操作
    let result = some_value.and_then(|x| {
        if x > 0 {
            Some(x * 2)
        } else {
            None
        }
    });
    println!("结果: {:?}", result); // 输出: Some(10)
}
```

## Result类型：处理可能失败的操作

`Result<T, E>`是Rust中处理可能失败的操作的标准方式。

```rust
enum Result<T, E> {
    Ok(T),   // 成功，包含值T
    Err(E),  // 失败，包含错误E
}
```

### 基本用法

```rust
use std::fs::File;
use std::io::Error;

fn read_file(filename: &str) -> Result<String, Error> {
    let file = File::open(filename)?;
    // 这里应该读取文件内容，为简化示例，只返回文件名
    Ok(format!("成功读取文件: {}", filename))
}

fn main() {
    match read_file("hello.txt") {
        Ok(content) => println!("{}", content),
        Err(e) => println!("读取文件失败: {}", e),
    }
}
```

### 错误传播操作符 `?`

`?`操作符是Rust中最方便的错误传播方式。

```rust
use std::fs::File;
use std::io::{self, Read};

fn read_username_from_file() -> Result<String, io::Error> {
    let mut file = File::open("username.txt")?;
    let mut username = String::new();
    file.read_to_string(&mut username)?;
    Ok(username)
}

// 等价于：
fn read_username_from_file_manual() -> Result<String, io::Error> {
    let mut file = match File::open("username.txt") {
        Ok(file) => file,
        Err(e) => return Err(e),
    };

    let mut username = String::new();
    match file.read_to_string(&mut username) {
        Ok(_) => Ok(username),
        Err(e) => Err(e),
    }
}
```

## 自定义错误类型

### 定义错误枚举

```rust
use std::fmt;

#[derive(Debug)]
enum AppError {
    Io(std::io::Error),
    Parse(std::num::ParseIntError),
    Network(String),
    Database(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::Io(err) => write!(f, "IO错误: {}", err),
            AppError::Parse(err) => write!(f, "解析错误: {}", err),
            AppError::Network(msg) => write!(f, "网络错误: {}", msg),
            AppError::Database(msg) => write!(f, "数据库错误: {}", msg),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Io(err) => Some(err),
            AppError::Parse(err) => Some(err),
            _ => None,
        }
    }
}
```

### 错误转换

```rust
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(err: std::num::ParseIntError) -> Self {
        AppError::Parse(err)
    }
}

// 现在可以使用?操作符自动转换错误
fn process_data() -> Result<i32, AppError> {
    let content = std::fs::read_to_string("data.txt")?;
    let number: i32 = content.trim().parse()?;
    Ok(number)
}
```

## 错误处理的最佳实践

### 1. 使用合适的错误类型

```rust
// 对于简单的错误，使用标准库类型
fn simple_function() -> Result<(), std::io::Error> {
    // ...
    Ok(())
}

// 对于复杂的应用，定义自定义错误类型
fn complex_function() -> Result<(), AppError> {
    // ...
    Ok(())
}
```

### 2. 错误上下文

```rust
use std::fs;
use std::path::Path;

fn read_config_file(path: &Path) -> Result<String, AppError> {
    fs::read_to_string(path)
        .map_err(|e| AppError::Network(format!("无法读取配置文件 {}: {}", path.display(), e)))
}
```

### 3. 错误处理策略

```rust
// 策略1: 处理错误
fn handle_errors() {
    match process_data() {
        Ok(result) => println!("处理结果: {}", result),
        Err(e) => eprintln!("处理失败: {}", e),
    }
}

// 策略2: 传播错误
fn propagate_errors() -> Result<(), AppError> {
    let result = process_data()?;
    println!("处理结果: {}", result);
    Ok(())
}

// 策略3: panic!（仅在不可恢复的错误时使用）
fn unrecoverable_error() {
    panic!("这是一个不可恢复的错误！");
}
```

## 实战例子：配置文件解析器

```rust
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use serde_json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON解析错误: {0}")]
    Json(#[from] serde_json::Error),

    #[error("配置文件未找到")]
    NotFound,

    #[error("无效的配置值: {field}")]
    InvalidValue { field: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub debug: bool,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(ConfigError::NotFound);
        }

        let content = fs::read_to_string(path)?;
        let config: AppConfig = serde_json::from_str(&content)?;

        // 验证配置
        if config.server.workers == 0 {
            return Err(ConfigError::InvalidValue {
                field: "server.workers".to_string(),
            });
        }

        if config.database.port == 0 {
            return Err(ConfigError::InvalidValue {
                field: "database.port".to_string(),
            });
        }

        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if !self.database.host.contains('.') {
            return Err(ConfigError::InvalidValue {
                field: "database.host".to_string(),
            });
        }

        if self.server.port > 65535 {
            return Err(ConfigError::InvalidValue {
                field: "server.port".to_string(),
            });
        }

        Ok(())
    }
}

fn load_and_validate_config() -> Result<(), ConfigError> {
    let config = AppConfig::from_file("config.json")?;
    config.validate()?;

    println!("配置加载成功:");
    println!("数据库: {}:{}", config.database.host, config.database.port);
    println!("服务器: {}:{}", config.server.host, config.server.port);

    Ok(())
}

fn main() {
    match load_and_validate_config() {
        Ok(()) => println!("配置验证成功！"),
        Err(e) => eprintln!("配置错误: {}", e),
    }
}
```

## 错误处理的模式对比

### 传统的异常处理 vs Rust的错误处理

**Python的异常处理：**
```python
def process_data():
    try:
        file = open("data.txt")
        content = file.read()
        number = int(content)
        return number * 2
    except FileNotFoundError:
        print("文件未找到")
        return 0
    except ValueError:
        print("无法解析为数字")
        return 0
    except Exception as e:
        print(f"未知错误: {e}")
        return 0
```

**Rust的错误处理：**
```rust
fn process_data() -> Result<i32, AppError> {
    let content = fs::read_to_string("data.txt")?;
    let number: i32 = content.trim().parse()?;
    Ok(number * 2)
}

fn main() {
    match process_data() {
        Ok(result) => println!("结果: {}", result),
        Err(AppError::Io(_)) => println!("文件未找到"),
        Err(AppError::Parse(_)) => println!("无法解析为数字"),
        Err(e) => println!("其他错误: {}", e),
    }
}
```

## 高级错误处理技巧

### 1. 使用`anyhow`库简化错误处理

```rust
use anyhow::{Context, Result};

fn read_file_with_context(path: &str) -> Result<String> {
    std::fs::read_to_string(path)
        .with_context(|| format!("无法读取文件: {}", path))
}

fn process_file() -> Result<()> {
    let content = read_file_with_context("data.txt")?;
    println!("文件内容: {}", content);
    Ok(())
}
```

### 2. 使用`thiserror`库定义错误类型

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("无效的输入: {input}")]
    InvalidInput { input: String },

    #[error("网络连接失败")]
    NetworkError,

    #[error("权限不足")]
    PermissionDenied,
}
```

### 3. 错误的链式处理

```rust
fn complex_operation() -> Result<String, AppError> {
    let step1_result = step1()
        .map_err(|e| AppError::Network(format!("步骤1失败: {}", e)))?;

    let step2_result = step2(step1_result)
        .map_err(|e| AppError::Database(format!("步骤2失败: {}", e)))?;

    Ok(step2_result)
}
```

## 总结

今天我们学习了：
- **Option类型**：处理可能为空的值
- **Result类型**：处理可能失败的操作
- **错误传播**：使用`?`操作符
- **自定义错误类型**：定义应用特定的错误
- **错误处理策略**：何时处理、何时传播、何时panic
- **最佳实践**：如何优雅地处理错误

**关键点：**
- Rust的错误处理是显式的，编译器强制处理错误
- 错误是值，不是异常，这使得错误处理更加可预测
- 使用合适的抽象层次来处理错误
- 在库代码中返回Result，在应用代码中根据需要处理错误

**明天预告：** 泛型和生命周期——Rust的抽象能力！

## 练习作业

1. 实现一个简单的命令行工具，处理文件操作和可能的错误
2. 创建一个网络客户端，处理各种网络错误
3. 实现一个配置管理系统，使用自定义错误类型

---

*如果你是C++开发者，重点关注Rust的错误处理相比异常的优势；如果你是Python开发者，重点关注Rust如何通过类型系统在编译时捕获错误。*