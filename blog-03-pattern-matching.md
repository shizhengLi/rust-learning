# 第三天：模式匹配——Rust的超能力

## 什么是模式匹配？

模式匹配是Rust最强大的特性之一。它比传统的`if-else`和`switch`语句更加强大和灵活。

### 从Python/C++的角度对比

**Python的条件匹配（3.10+）：**
```python
# Python 3.10+
def process_value(value):
    match value:
        case 1:
            print("One")
        case 2:
            print("Two")
        case _:
            print("Something else")
```

**C++的switch语句：**
```cpp
void process_value(int value) {
    switch (value) {
        case 1:
            std::cout << "One" << std::endl;
            break;
        case 2:
            std::cout << "Two" << std::endl;
            break;
        default:
            std::cout << "Something else" << std::endl;
    }
}
```

**Rust的模式匹配：**
```rust
fn process_value(value: i32) {
    match value {
        1 => println!("One"),
        2 => println!("Two"),
        _ => println!("Something else"),
    }
}
```

## match表达式的基础用法

### 基本语法

```rust
fn main() {
    let number = 13;

    match number {
        1 => println!("One"),
        2 | 3 | 5 | 7 | 11 => println!("这是一个质数"),
        13..=19 => println!("一个青少年"),
        _ => println!("其他数字"),
    }
}
```

### match作为表达式

```rust
fn main() {
    let x = 5;
    let message = match x {
        0 => "零",
        1 => "一",
        2 => "二",
        _ => "其他",
    };

    println!("x是{}", message);
}
```

## 深入模式匹配

### 解构元组

```rust
fn main() {
    let tuple = (1, "hello", 3.14);

    match tuple {
        (1, _, _) => println!("第一个元素是1"),
        (_, text, _) => println!("第二个元素是{}", text),
        (x, y, z) => println!("所有元素：{}, {}, {}", x, y, z),
    }
}
```

### 解构枚举

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}

fn process_message(msg: Message) {
    match msg {
        Message::Quit => println!("退出"),
        Message::Move { x, y } => println!("移动到({}, {})", x, y),
        Message::Write(text) => println!("写入: {}", text),
        Message::ChangeColor(r, g, b) => println!("颜色RGB({}, {}, {})", r, g, b),
    }
}
```

### 解构结构体

```rust
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let p = Point { x: 0, y: 7 };

    match p {
        Point { x, y: 0 } => println!("在x轴上，x={}", x),
        Point { x: 0, y } => println!("在y轴上，y={}", y),
        Point { x, y } => println!("在({}, {})", x, y),
    }
}
```

## 模式的高级特性

### 使用`@`绑定

```rust
fn main() {
    let message = Message::Write("Hello".to_string());

    match message {
        Message::Write(text @ _) => println!("写入消息: {}", text),
        _ => println!("其他消息"),
    }
}
```

### 使用`..`忽略剩余值

```rust
fn main() {
    let numbers = (1, 2, 3, 4, 5);

    match numbers {
        (first, .., last) => println!("第一个: {}, 最后一个: {}", first, last),
    }
}
```

### 守卫（Guards）

```rust
fn main() {
    let x = 4;

    match x {
        n if n % 2 == 0 => println!("偶数: {}", n),
        n => println!("奇数: {}", n),
    }
}
```

## if let和while let

### if let - 简化的匹配

```rust
fn main() {
    let some_value = Some(3u8);

    // 完整的match
    match some_value {
        Some(3) => println!("三个"),
        _ => (),
    }

    // 使用if let
    if let Some(3) = some_value {
        println!("三个");
    }
}
```

### while let - 循环匹配

```rust
fn main() {
    let mut stack = Vec::new();

    stack.push(1);
    stack.push(2);
    stack.push(3);

    while let Some(top) = stack.pop() {
        println!("{}", top);
    }
}
```

## 控制流的其他特性

### loop循环

```rust
fn main() {
    let mut counter = 0;

    let result = loop {
        counter += 1;

        if counter == 10 {
            break counter * 2;  // 从循环返回值
        }
    };

    println!("结果是: {}", result);
}
```

### while循环

```rust
fn main() {
    let mut number = 3;

    while number != 0 {
        println!("{}", number);
        number -= 1;
    }

    println!("发射！");
}
```

### for循环

```rust
fn main() {
    let a = [10, 20, 30, 40, 50];

    for element in a.iter() {
        println!("值是: {}", element);
    }

    // 使用范围
    for number in (1..4).rev() {
        println!("{}!", number);
    }
    println!("发射！");
}
```

## 实战例子：解析命令行参数

```rust
use std::env;

enum Command {
    Help,
    Version,
    Count { file: String, lines: bool },
    Unknown(String),
}

fn parse_command(args: Vec<String>) -> Command {
    match args.as_slice() {
        [_, cmd] if cmd == "--help" || cmd == "-h" => Command::Help,
        [_, cmd] if cmd == "--version" || cmd == "-v" => Command::Version,
        [_, cmd, file] if cmd == "--count" || cmd == "-c" => {
            Command::Count {
                file: file.clone(),
                lines: false,
            }
        }
        [_, cmd, file] if cmd == "--count-lines" || cmd == "-l" => {
            Command::Count {
                file: file.clone(),
                lines: true,
            }
        }
        [_, unknown] => Command::Unknown(unknown.clone()),
        _ => Command::Help,
    }
}

fn execute_command(cmd: Command) {
    match cmd {
        Command::Help => {
            println!("用法：");
            println!("  program --help          显示帮助");
            println!("  program --version       显示版本");
            println!("  program --count <file>  计算字符数");
            println!("  program --count-lines <file>  计算行数");
        }
        Command::Version => println!("版本 1.0.0"),
        Command::Count { file, lines } => {
            println!("{} {} {}", if lines { "计算行数" } else { "计算字符数" }, "在文件", file);
        }
        Command::Unknown(unknown) => {
            println!("未知命令: {}", unknown);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = parse_command(args);
    execute_command(command);
}
```

## 深入理解：模式匹配的类型安全

### 穷尽性检查

Rust的match必须穷尽所有可能：

```rust
enum Direction {
    North,
    South,
    East,
    West,
}

fn direction_to_string(dir: Direction) -> String {
    match dir {
        Direction::North => "北".to_string(),
        Direction::South => "南".to_string(),
        Direction::East => "东".to_string(),
        // 编译错误！没有处理West
    }
}
```

### 不可反驳的模式

某些上下文要求模式必须是不可反驳的：

```rust
fn main() {
    let x = 5;
    let Some(y) = Some(x);  // 编译错误！Some(y)可能是反驳的

    if let Some(y) = Some(x) {  // OK，if let允许反驳的模式
        println!("{}", y);
    }
}
```

## 实战例子：简单的状态机

```rust
#[derive(Debug)]
enum TrafficLight {
    Red,
    Yellow,
    Green,
}

impl TrafficLight {
    fn next(self) -> Self {
        match self {
            TrafficLight::Red => TrafficLight::Green,
            TrafficLight::Yellow => TrafficLight::Red,
            TrafficLight::Green => TrafficLight::Yellow,
        }
    }

    fn duration(&self) -> u32 {
        match self {
            TrafficLight::Red => 30,
            TrafficLight::Yellow => 5,
            TrafficLight::Green => 25,
        }
    }
}

fn main() {
    let mut light = TrafficLight::Red;

    for _ in 0..10 {
        println!("当前状态: {:?}, 持续时间: {}秒", light, light.duration());
        light = light.next();
    }
}
```

## 总结

今天我们学习了：
- **match表达式**：Rust强大的模式匹配
- **解构**：从复杂数据结构中提取值
- **if let和while let**：简化的匹配语法
- **循环结构**：loop、while、for
- **模式匹配的类型安全**：穷尽性检查

**关键点：**
- 模式匹配比传统的条件语句更强大
- 编译器会检查是否处理了所有情况
- 模式可以解构复杂数据结构
- Rust的控制流既安全又灵活

**明天预告：** 结构体、枚举和特质——Rust的面向对象编程！

## 练习作业

1. 实现一个简单的计算器，使用模式匹配处理不同的运算符
2. 创建一个温度转换程序，处理摄氏度、华氏度、开尔文温度
3. 编写一个简单的解析器，解析类似"1 + 2 * 3"的表达式

---

*如果你是C++开发者，重点关注Rust模式匹配相比switch的优势；如果你是Python开发者，重点关注Rust模式匹配的类型安全和编译时检查。*