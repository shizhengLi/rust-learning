# 从Python/C++到Rust：第一天——初识Rust与环境搭建

## 为什么选择Rust？

作为一个有Python和C++背景的开发者，你可能会问：为什么还要学习Rust？

### 从Python开发者的角度

Python以其简洁的语法和强大的生态系统著称，但也存在一些痛点：
- **性能问题**：Python的执行速度相对较慢
- **GIL限制**：无法充分利用多核CPU
- **运行时错误**：很多错误只能在运行时发现

Rust提供了：
- **C级别的性能**：无需牺牲开发体验
- **真正的并发**：没有GIL的限制
- **编译时安全**：在编译阶段就发现大部分错误

### 从C++开发者的角度

C++提供了极致的性能和控制权，但代价是：
- **内存安全问题**：悬垂指针、缓冲区溢出等
- **复杂的语法**：学习曲线陡峭
- **手动内存管理**：容易出错且繁琐

Rust的解决方案：
- **零成本抽象**：高性能的同时保证安全
- **现代化语法**：更清晰的表达方式
- **自动内存管理**：无需GC，无需手动delete

## 安装Rust环境

### 1. 安装rustup

Rust的版本管理工具rustup是安装Rust的首选方式：

```bash
# 在Unix系统（macOS、Linux）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 在Windows
# 下载并运行 https://win.rustup.rs/
```

### 2. 验证安装

```bash
rustc --version
cargo --version
```

### 3. 配置开发环境

#### VS Code（推荐）
安装以下扩展：
- `rust-analyzer`：提供智能代码补全和错误检查
- `CodeLLDB`：调试支持

#### IntelliJ IDEA
安装Rust插件即可。

## 第一个Rust程序

### Hello World

```rust
// main.rs
fn main() {
    println!("Hello, Rust world!");
}
```

### 编译和运行

```bash
# 编译
rustc main.rs

# 运行
./main
```

### 使用Cargo（Rust的构建工具）

```bash
# 创建新项目
cargo new hello_rust
cd hello_rust

# 运行
cargo run

# 检查编译（不生成可执行文件）
cargo check

# 构建发布版本
cargo build --release
```

## Rust基础语法对比

### 变量声明

```rust
// Rust - 不可变变量（默认）
let x = 5;
// x = 6; // 编译错误！

// 可变变量
let mut y = 10;
y = 15; // OK

// 常量
const MAX_POINTS: u32 = 100_000;
```

**对比Python：**
```python
# Python - 变量默认可变
x = 5
x = 6  # OK

MAX_POINTS = 100000  # 约定，但非强制
```

**对比C++：**
```cpp
// C++
const int MAX_POINTS = 100000;
int x = 5;
x = 6; // OK

// C++11的不可变变量
const int y = 10;
// y = 15; // 编译错误
```

### 函数

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b  // 无需return，最后一个表达式就是返回值
}

fn main() {
    let result = add(5, 3);
    println!("5 + 3 = {}", result);
}
```

### 基本数据类型

```rust
// 整数
let decimal = 98_222;      // 十进制
let hex = 0xff;            // 十六进制
let octal = 0o77;          // 八进制
let binary = 0b1111_0000;  // 二进制
let byte = b'A';           // 字节（u8）

// 浮点数
let x = 2.0;  // f64
let y: f32 = 3.0;  // f32

// 布尔值
let t = true;
let f: bool = false;

// 字符
let c = 'z';
let z = 'ℤ';
let heart_eyed_cat = '😻';
```

## 练习：编写一个简单的计算器

```rust
use std::io;

fn main() {
    println!("简单计算器");
    println!("输入第一个数字：");

    let mut num1 = String::new();
    io::stdin().read_line(&mut num1).unwrap();
    let num1: f64 = num1.trim().parse().unwrap();

    println!("输入运算符（+、-、*、/）：");
    let mut operator = String::new();
    io::stdin().read_line(&mut operator).unwrap();
    let operator = operator.trim();

    println!("输入第二个数字：");
    let mut num2 = String::new();
    io::stdin().read_line(&mut num2).unwrap();
    let num2: f64 = num2.trim().parse().unwrap();

    let result = match operator {
        "+" => num1 + num2,
        "-" => num1 - num2,
        "*" => num1 * num2,
        "/" => num1 / num2,
        _ => {
            println!("未知运算符");
            return;
        }
    };

    println!("{} {} {} = {}", num1, operator, num2, result);
}
```

## 总结

今天我们：
- 了解了Rust相对于Python和C++的优势
- 搭建了Rust开发环境
- 写了第一个Rust程序
- 对比了基础语法差异

**明天预告：** Rust的核心特性——所有权系统，这是Rust最重要的创新！

## 练习作业

1. 修改计算器程序，添加更多的运算符（如幂运算、取模）
2. 编写一个程序，打印Fibonacci数列的前20项
3. 尝试使用`if`和`match`语句实现简单的猜数字游戏

---

*这篇博客适合有Python/C++基础的初学者。如果你是Python开发者，重点关注Rust的类型系统和编译时检查；如果你是C++开发者，重点关注Rust的内存安全保证。*