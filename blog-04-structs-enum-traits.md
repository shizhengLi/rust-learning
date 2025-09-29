# 第四天：结构体、枚举和特质——Rust的类型系统

## 结构体（Structs）：自定义数据类型

结构体让你能够创建自定义的数据类型，将相关数据组合在一起。

### 从C++的角度理解

**C++的结构体：**
```cpp
#include <string>

struct Person {
    std::string name;
    int age;
    double height;
};

// 使用
Person p = {"Alice", 30, 1.65};
std::cout << p.name << std::endl;
```

**Rust的结构体：**
```rust
struct Person {
    name: String,
    age: u32,
    height: f64,
}

fn main() {
    let p = Person {
        name: String::from("Alice"),
        age: 30,
        height: 1.65,
    };
    println!("{}", p.name);
}
```

### 从Python的角度理解

**Python的类：**
```python
class Person:
    def __init__(self, name: str, age: int, height: float):
        self.name = name
        self.age = age
        self.height = height

# 使用
p = Person("Alice", 30, 1.65)
print(p.name)
```

### 结构体的类型

#### 1. 经典结构体（C-style struct）

```rust
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    // 关联函数（类似静态方法）
    fn new(width: u32, height: u32) -> Self {
        Rectangle { width, height }
    }

    // 方法（需要&self）
    fn area(&self) -> u32 {
        self.width * self.height
    }

    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
}

fn main() {
    let rect1 = Rectangle::new(30, 50);
    let rect2 = Rectangle { width: 10, height: 40 };

    println!("矩形1的面积: {}", rect1.area());
    println!("矩形1能容纳矩形2吗? {}", rect1.can_hold(&rect2));
}
```

#### 2. 元组结构体（Tuple struct）

```rust
struct Color(i32, i32, i32);
struct Point(i32, i32, i32);

fn main() {
    let black = Color(0, 0, 0);
    let origin = Point(0, 0, 0);

    let Color(r, g, b) = black;  // 解构
    println!("RGB: {}, {}, {}", r, g, b);
}
```

#### 3. 单元结构体（Unit-like struct）

```rust
struct AlwaysEqual;

fn main() {
    let subject = AlwaysEqual;
    // 用于实现某些特质，但不存储数据
}
```

## 枚举（Enums）：定义所有可能的值

枚举让你能够定义一个类型，它可以有多个不同的变体。

### 从Python的角度理解

**Python的枚举：**
```python
from enum import Enum

class Message(Enum):
    QUIT = 1
    MOVE = 2
    WRITE = 3

def process_message(msg: Message):
    if msg == Message.QUIT:
        print("退出")
    elif msg == Message.MOVE:
        print("移动")
```

**Rust的枚举：**
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

### 常用枚举

#### Option枚举

```rust
enum Option<T> {
    Some(T),
    None,
}

fn main() {
    let some_number = Some(5);
    let some_char = Some('e');
    let absent_number: Option<i32> = None;

    // 使用模式匹配
    match some_number {
        Some(n) => println!("有数字: {}", n),
        None => println!("没有数字"),
    }

    // 使用if let
    if let Some(n) = some_number {
        println!("数字是: {}", n);
    }
}
```

#### Result枚举

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}

fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err(String::from("除数不能为零"))
    } else {
        Ok(a / b)
    }
}

fn main() {
    match divide(10.0, 2.0) {
        Ok(result) => println!("结果: {}", result),
        Err(e) => println!("错误: {}", e),
    }

    match divide(10.0, 0.0) {
        Ok(result) => println!("结果: {}", result),
        Err(e) => println!("错误: {}", e),
    }
}
```

## 特质（Traits）：定义共享行为

特质定义了一组方法，可以被不同的类型实现。这类似于其他语言中的接口。

### 从C++的角度理解

**C++的抽象类/接口：**
```cpp
class Drawable {
public:
    virtual void draw() const = 0;
    virtual ~Drawable() = default;
};

class Circle : public Drawable {
public:
    void draw() const override {
        std::cout << "绘制圆形" << std::endl;
    }
};
```

**Rust的特质：**
```rust
trait Drawable {
    fn draw(&self);
}

struct Circle {
    radius: f64,
}

impl Drawable for Circle {
    fn draw(&self) {
        println!("绘制圆形，半径: {}", self.radius);
    }
}

struct Rectangle {
    width: f64,
    height: f64,
}

impl Drawable for Rectangle {
    fn draw(&self) {
        println!("绘制矩形，宽: {}, 高: {}", self.width, self.height);
    }
}

fn draw_object<T: Drawable>(object: &T) {
    object.draw();
}

fn main() {
    let circle = Circle { radius: 5.0 };
    let rectangle = Rectangle { width: 3.0, height: 4.0 };

    draw_object(&circle);
    draw_object(&rectangle);
}
```

### 从Python的角度理解

**Python的鸭子类型：**
```python
class Drawable:
    def draw(self):
        pass

class Circle(Drawable):
    def __init__(self, radius):
        self.radius = radius

    def draw(self):
        print(f"绘制圆形，半径: {self.radius}")

def draw_object(obj: Drawable):
    obj.draw()
```

### 特质的语法糖

```rust
// 完整语法
fn draw_object<T: Drawable>(object: &T) {
    object.draw();
}

// 使用where子句
fn draw_object2<T>(object: &T)
where
    T: Drawable,
{
    object.draw();
}

// 使用impl Trait（Rust 2018+）
fn draw_object3(object: &impl Drawable) {
    object.draw();
}
```

### 默认实现

```rust
trait Summary {
    fn summarize(&self) -> String {
        String::from("(阅读更多...)")
    }

    fn summarize_author(&self) -> String;
}

struct Article {
    title: String,
    author: String,
    content: String,
}

impl Summary for Article {
    fn summarize_author(&self) -> String {
        format!("@{}", self.author)
    }

    // 可以选择覆盖默认实现
    fn summarize(&self) -> String {
        format!("{} - {}", self.title, self.summarize_author())
    }
}

fn main() {
    let article = Article {
        title: String::from("Rust入门"),
        author: String::from("张三"),
        content: String::from("Rust是一种系统编程语言..."),
    };

    println!("{}", article.summarize());
}
```

### 特质作为参数

```rust
fn notify(item: &impl Summary) {
    println!("Breaking news! {}", item.summarize());
}

// 或者使用泛型
fn notify2<T: Summary>(item: &T) {
    println!("Breaking news! {}", item.summarize());
}

// 多个特质约束
fn some_function<T: Summary + Display>(t: &T) {
    println!("Breaking news! {}", t.summarize());
    println!("Display: {}", t);
}
```

## 实战例子：形状计算器

```rust
use std::f64::consts::PI;

trait Shape {
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
    fn describe(&self) -> String {
        format!("面积: {}, 周长: {}", self.area(), self.perimeter())
    }
}

struct Circle {
    radius: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        PI * self.radius * self.radius
    }

    fn perimeter(&self) -> f64 {
        2.0 * PI * self.radius
    }
}

struct Rectangle {
    width: f64,
    height: f64,
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }

    fn perimeter(&self) -> f64 {
        2.0 * (self.width + self.height)
    }
}

struct Triangle {
    base: f64,
    height: f64,
    side1: f64,
    side2: f64,
    side3: f64,
}

impl Shape for Triangle {
    fn area(&self) -> f64 {
        0.5 * self.base * self.height
    }

    fn perimeter(&self) -> f64 {
        self.side1 + self.side2 + self.side3
    }
}

fn print_shape_info<T: Shape>(shape: &T) {
    println!("{}", shape.describe());
}

fn main() {
    let circle = Circle { radius: 5.0 };
    let rectangle = Rectangle { width: 4.0, height: 3.0 };
    let triangle = Triangle {
        base: 3.0,
        height: 4.0,
        side1: 3.0,
        side2: 4.0,
        side3: 5.0,
    };

    print_shape_info(&circle);
    print_shape_info(&rectangle);
    print_shape_info(&triangle);
}
```

## 深入理解：特质对象

### 动态分发 vs 静态分发

```rust
// 静态分发（编译时确定）
fn draw_static<T: Drawable>(object: &T) {
    object.draw();
}

// 动态分发（运行时确定）
fn draw_dynamic(object: &dyn Drawable) {
    object.draw();
}

fn main() {
    let shapes: Vec<Box<dyn Shape>> = vec![
        Box::new(Circle { radius: 5.0 }),
        Box::new(Rectangle { width: 4.0, height: 3.0 }),
    ];

    for shape in shapes {
        println!("{}", shape.describe());
    }
}
```

## 实战例子：简单的图形用户界面

```rust
trait Widget {
    fn draw(&self);
    fn handle_event(&self, event: Event);
}

struct Button {
    label: String,
    on_click: Box<dyn Fn()>,
}

impl Widget for Button {
    fn draw(&self) {
        println!("[按钮: {}]", self.label);
    }

    fn handle_event(&self, event: Event) {
        match event {
            Event::Click => (self.on_click)(),
            _ => {}
        }
    }
}

struct Label {
    text: String,
}

impl Widget for Label {
    fn draw(&self) {
        println!("标签: {}", self.text);
    }

    fn handle_event(&self, _event: Event) {
        // 标签不处理事件
    }
}

enum Event {
    Click,
    Hover,
    KeyPress(char),
}

struct Window {
    widgets: Vec<Box<dyn Widget>>,
}

impl Window {
    fn new() -> Self {
        Window { widgets: Vec::new() }
    }

    fn add_widget(&mut self, widget: Box<dyn Widget>) {
        self.widgets.push(widget);
    }

    fn draw(&self) {
        println!("=== 窗口 ===");
        for widget in &self.widgets {
            widget.draw();
        }
        println!("==========");
    }

    fn handle_event(&self, event: Event) {
        for widget in &self.widgets {
            widget.handle_event(event.clone());
        }
    }
}

fn main() {
    let mut window = Window::new();

    window.add_widget(Box::new(Label {
        text: "请点击按钮:".to_string(),
    }));

    window.add_widget(Box::new(Button {
        label: "确定".to_string(),
        on_click: Box::new(|| println!("按钮被点击了！")),
    }));

    window.draw();
    window.handle_event(Event::Click);
}
```

## 总结

今天我们学习了：
- **结构体**：组织相关数据的自定义类型
- **枚举**：定义类型的所有可能值
- **特质**：定义共享行为的接口
- **特质对象**：实现运行时多态

**关键点：**
- Rust的类型系统既强大又安全
- 特质提供了零成本的抽象
- 枚举+模式匹配提供了强大的表达能力
- 结构体+特质实现了面向对象编程

**明天预告：** 错误处理——Result和Option的深入探讨！

## 练习作业

1. 实现一个`Vehicle`特质，为`Car`、`Bicycle`、`Airplane`实现该特质
2. 创建一个`Library`系统，使用枚举表示不同类型的媒体（书籍、电影、音乐）
3. 实现一个简单的表达式求值器，使用枚举表示不同的操作符和操作数

---

*如果你是C++开发者，重点关注Rust的特质如何替代C++的虚函数和抽象类；如果你是Python开发者，重点关注Rust的强类型系统和编译时检查如何提供更高的安全性。*