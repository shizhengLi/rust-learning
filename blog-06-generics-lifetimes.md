# 第六天：泛型和生命周期——Rust的抽象能力

## 泛型：编写更灵活的代码

泛型是Rust中实现代码重用和抽象的重要工具。它允许你编写可以处理多种类型的代码，同时保持类型安全。

### 从C++的角度理解

**C++的模板：**
```cpp
template<typename T>
T add(T a, T b) {
    return a + b;
}

// 使用
int result = add(5, 3);        // T = int
double result2 = add(2.5, 1.5); // T = double
```

**Rust的泛型：**
```rust
fn add<T: std::ops::Add<Output = T>>(a: T, b: T) -> T {
    a + b
}

// 使用
let result = add(5, 3);        // T = i32
let result2 = add(2.5, 1.5);   // T = f64
```

### 从Python的角度理解

**Python的鸭子类型：**
```python
def add(a, b):
    return a + b

# 使用
result = add(5, 3)        # 可以工作
result2 = add(2.5, 1.5)   # 可以工作
result3 = add("hello", " world")  # 也可以工作
```

## 泛型的基础用法

### 函数中的泛型

```rust
// 简单的泛型函数
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];

    for item in list {
        if item > largest {
            largest = item;
        }
    }

    largest
}

fn main() {
    let numbers = vec![34, 50, 25, 100, 65];
    let chars = vec!['y', 'm', 'a', 'q'];

    println!("最大的数字: {}", largest(&numbers));
    println!("最大的字符: {}", largest(&chars));
}
```

### 结构体中的泛型

```rust
struct Point<T> {
    x: T,
    y: T,
}

impl<T> Point<T> {
    fn new(x: T, y: T) -> Self {
        Point { x, y }
    }

    fn get_x(&self) -> &T {
        &self.x
    }
}

// 为特定类型实现方法
impl Point<f32> {
    fn distance_from_origin(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}

fn main() {
    let integer_point = Point::new(5, 10);
    let float_point = Point::new(3.0, 4.0);

    println!("x坐标: {}", integer_point.get_x());
    println!("距离原点: {}", float_point.distance_from_origin());
}
```

### 多个泛型参数

```rust
struct Pair<T, U> {
    first: T,
    second: U,
}

impl<T, U> Pair<T, U> {
    fn new(first: T, second: U) -> Self {
        Pair { first, second }
    }
}

fn main() {
    let pair1 = Pair::new(1, "one");
    let pair2 = Pair::new("hello", 3.14);

    println!("Pair 1: {}, {}", pair1.first, pair1.second);
    println!("Pair 2: {}, {}", pair2.first, pair2.second);
}
```

## 特质约束（Trait Bounds）

特质约束限制了泛型类型必须实现特定的特质。

### 基本语法

```rust
fn display<T: std::fmt::Display>(item: T) {
    println!("{}", item);
}

fn compare<T: PartialEq>(a: T, b: T) -> bool {
    a == b
}

fn main() {
    display(42);
    display("Hello");

    println!("相等: {}", compare(5, 5));
    println!("相等: {}", compare("hello", "world"));
}
```

### where子句

当约束很多时，使用where子句更清晰：

```rust
// 复杂的约束
fn some_function<T: Display + Clone, U: Clone + Debug>(t: T, u: U) -> i32 {
    // ...
    0
}

// 使用where子句更清晰
fn some_function_where<T, U>(t: T, u: U) -> i32
where
    T: Display + Clone,
    U: Clone + Debug,
{
    // ...
    0
}
```

### 多个约束和实现

```rust
use std::fmt::{Debug, Display};

trait Summary {
    fn summarize(&self) -> String;
}

struct NewsArticle {
    title: String,
    content: String,
}

impl Summary for NewsArticle {
    fn summarize(&self) -> String {
        format!("{}...", self.title)
    }
}

struct Tweet {
    username: String,
    content: String,
}

impl Summary for Tweet {
    fn summarize(&self) -> String {
        format!("{}: {}", self.username, self.content)
    }
}

// 通知函数，要求类型实现了Summary和Display特质
fn notify<T: Summary + Display>(item: T) {
    println!("新闻: {}", item.summarize());
    println!("详细: {}", item);
}

fn main() {
    let article = NewsArticle {
        title: "Rust发布新版本".to_string(),
        content: "Rust 1.70发布了".to_string(),
    };

    let tweet = Tweet {
        username: "rustlang".to_string(),
        content: "Rust很棒！".to_string(),
    };

    // 注意：NewsArticle没有实现Display，所以不能使用notify
    // notify(article); // 编译错误！

    // Tweet需要实现Display才能使用notify
}
```

## 生命周期：确保引用的有效性

生命周期是Rust中防止悬垂引用的重要机制。它告诉编译器引用有效的时间范围。

### 生命周期标注

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

fn main() {
    let string1 = String::from("abcd");
    let string2 = "xyz";

    let result = longest(string1.as_str(), string2);
    println!("最长的字符串是: {}", result);
}
```

### 结构体中的生命周期

```rust
struct Book<'a> {
    title: &'a str,
    author: &'a str,
}

impl<'a> Book<'a> {
    fn new(title: &'a str, author: &'a str) -> Self {
        Book { title, author }
    }

    fn get_description(&self) -> String {
        format!("《{}》by {}", self.title, self.author)
    }
}

fn main() {
    let title = String::from("Rust编程");
    let author = String::from("Steve Klabnik");

    {
        let book = Book::new(&title, &author);
        println!("{}", book.get_description());
    } // book在这里离开作用域
}
```

### 生命周期省略规则

Rust有一些规则可以自动推断生命周期：

```rust
// 这些函数是等价的
fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();
    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return &s[0..i];
        }
    }
    &s[..]
}

// 显式生命周期标注
fn first_word_explicit<'a>(s: &'a str) -> &'a str {
    let bytes = s.as_bytes();
    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return &s[0..i];
        }
    }
    &s[..]
}
```

### 静态生命周期

`'static`生命周期表示整个程序的运行时间：

```rust
fn get_static_string() -> &'static str {
    "这是一个静态字符串"
}

fn main() {
    let s: &'static str = "Hello, world!";
    let s2 = get_static_string();

    println!("{}", s);
    println!("{}", s2);
}
```

## 高级泛型特性

### 关联类型

```rust
trait Iterator {
    type Item;  // 关联类型

    fn next(&mut self) -> Option<Self::Item>;
}

struct Counter {
    count: u32,
}

impl Iterator for Counter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.count += 1;
        if self.count < 6 {
            Some(self.count)
        } else {
            None
        }
    }
}

fn main() {
    let mut counter = Counter { count: 0 };

    while let Some(value) = counter.next() {
        println!("{}", value);
    }
}
```

### 泛型特质

```rust
trait From<T> {
    fn from(T) -> Self;
}

// String实现了From<&str>
let s = String::from("hello");

// 使用into()方法
let s2: String = "hello".into();
```

### 默认泛型参数

```rust
struct Point<T = i32> {
    x: T,
    y: T,
}

impl<T> Point<T> {
    fn new(x: T, y: T) -> Self {
        Point { x, y }
    }
}

fn main() {
    let p1 = Point::new(1, 2);        // T = i32
    let p2 = Point::new(1.0, 2.0);    // T = f64
}
```

## 实战例子：泛型数据结构

### 链表实现

```rust
use std::rc::{Rc, Weak};

#[derive(Debug)]
struct Node<T> {
    data: T,
    next: Option<Rc<Node<T>>>,
    prev: Option<Weak<Node<T>>>,
}

#[derive(Debug)]
struct LinkedList<T> {
    head: Option<Rc<Node<T>>>,
    tail: Option<Weak<Node<T>>>,
    length: usize,
}

impl<T> LinkedList<T> {
    fn new() -> Self {
        LinkedList {
            head: None,
            tail: None,
            length: 0,
        }
    }

    fn push_front(&mut self, data: T) {
        let new_node = Rc::new(Node {
            data,
            next: self.head.take(),
            prev: None,
        });

        if let Some(head) = &self.head {
            let mut head_mut = Rc::clone(head);
            // 这里需要修改prev，但由于Rc的限制，这比较复杂
            // 实际实现可能需要使用RefCell
        }

        self.head = Some(new_node);
        self.length += 1;
    }

    fn len(&self) -> usize {
        self.length
    }
}

fn main() {
    let mut list = LinkedList::new();
    list.push_front(1);
    list.push_front(2);
    list.push_front(3);

    println!("链表长度: {}", list.len());
}
```

### 泛型缓存实现

```rust
use std::collections::HashMap;
use std::hash::Hash;

struct Cache<K, V> {
    data: HashMap<K, V>,
    capacity: usize,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn new(capacity: usize) -> Self {
        Cache {
            data: HashMap::new(),
            capacity,
        }
    }

    fn get(&self, key: &K) -> Option<V> {
        self.data.get(key).cloned()
    }

    fn set(&mut self, key: K, value: V) {
        if self.data.len() >= self.capacity {
            // 简单的LRU策略：移除第一个元素
            if let Some(first_key) = self.data.keys().next().cloned() {
                self.data.remove(&first_key);
            }
        }
        self.data.insert(key, value);
    }

    fn contains(&self, key: &K) -> bool {
        self.data.contains_key(key)
    }
}

fn main() {
    let mut cache = Cache::new(3);

    cache.set("key1".to_string(), "value1".to_string());
    cache.set("key2".to_string(), "value2".to_string());
    cache.set("key3".to_string(), "value3".to_string());

    println!("缓存包含key1: {}", cache.contains(&"key1".to_string()));
    println!("key1的值: {:?}", cache.get(&"key1".to_string()));

    // 添加第四个元素，会移除第一个
    cache.set("key4".to_string(), "value4".to_string());
    println!("缓存包含key1: {}", cache.contains(&"key1".to_string()));
    println!("缓存包含key4: {}", cache.contains(&"key4".to_string()));
}
```

## 深入理解：生命周期与借用检查器

### 复杂的生命周期例子

```rust
struct Context<'s> {
    data: &'s str,
}

struct Parser<'c, 's> {
    context: &'c Context<'s>,
}

impl<'c, 's> Parser<'c, 's> {
    fn parse(&self) -> Result<&str, String> {
        Ok(self.context.data)
    }
}

fn create_parser<'s>(context: &'s Context<'s>) -> Parser<'s, 's> {
    Parser { context }
}

fn main() {
    let data = String::from("解析数据");
    let context = Context { data: &data };
    let parser = create_parser(&context);

    match parser.parse() {
        Ok(result) => println!("解析结果: {}", result),
        Err(e) => println!("解析错误: {}", e),
    }
}
```

### 生命周期子类型化

```rust
fn get_first_word<'a, 'b>(s1: &'a str, s2: &'b str) -> &'a str
where
    'b: 'a,  // 'b必须比'a长
{
    if s1.len() < s2.len() {
        s1
    } else {
        s2
    }
}

fn main() {
    let s1 = String::from("short");
    let s2 = String::from("longer string");

    let result = get_first_word(&s1, &s2);
    println!("第一个单词: {}", result);
}
```

## 总结

今天我们学习了：
- **泛型**：编写灵活、可重用的代码
- **特质约束**：限制泛型类型的行为
- **生命周期**：确保引用的有效性
- **高级泛型特性**：关联类型、默认参数等
- **实际应用**：链表、缓存等数据结构

**关键点：**
- 泛型在编译时单态化，没有运行时开销
- 生命周期是编译时概念，不影响运行时性能
- 特质约束提供了强大的抽象能力
- Rust的泛型系统既灵活又安全

**明天预告：** 内存管理深入理解——智能指针和内存布局！

## 练习作业

1. 实现一个泛型的二叉搜索树
2. 创建一个泛型的观察者模式实现
3. 编写一个泛型的函数，可以序列化和反序列化不同的数据类型

---

*如果你是C++开发者，重点关注Rust泛型与C++模板的相似性和差异；如果你是Python开发者，重点关注Rust如何在保持灵活性的同时提供编译时类型安全。*