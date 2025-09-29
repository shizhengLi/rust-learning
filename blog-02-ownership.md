# 第二天：所有权系统——Rust的革命性创新

## 什么是所有权？

所有权（Ownership）是Rust最核心、最独特的特性。它让Rust在没有垃圾回收器的情况下，保证了内存安全。

### 从C++的角度理解

在C++中，我们经常遇到这样的问题：

```cpp
void use_after_free() {
    int* ptr = new int(42);
    delete ptr;
    *ptr = 10;  // 悬垂指针！未定义行为
}

void memory_leak() {
    int* ptr = new int(42);
    // 忘记delete，内存泄漏！
}
```

### 从Python的角度理解

Python使用垃圾回收器（GC）来管理内存：

```python
def python_memory_management():
    data = [1, 2, 3, 4, 5]  # 创建列表
    # 当data离开作用域，GC会自动回收内存
    # 但我们不知道什么时候会发生
```

## 所有权三规则

Rust的所有权系统遵循三个基本规则：

1. **每个值都有一个变量作为它的所有者**
2. **每个值同时只能有一个所有者**
3. **当所有者离开作用域时，值被丢弃**

### 移动（Move）语义

```rust
fn main() {
    let s1 = String::from("Hello");  // s1拥有字符串"Hello"
    let s2 = s1;                     // 所有权从s1移动到s2

    // println!("{}", s1);          // 编译错误！s1不再有效
    println!("{}", s2);              // OK
}
```

**对比C++：**
```cpp
#include <string>
#include <iostream>

void cpp_example() {
    std::string s1 = "Hello";
    std::string s2 = s1;  // 复制！两个字符串都有效
    std::cout << s1 << std::endl;  // OK
    std::cout << s2 << std::endl;  // OK
}
```

**对比Python：**
```python
def python_example():
    s1 = "Hello"
    s2 = s1  # 引用同一个对象
    print(s1)  # OK
    print(s2)  # OK
```

### 克隆（Clone）

如果需要深度复制，使用`clone`：

```rust
fn main() {
    let s1 = String::from("Hello");
    let s2 = s1.clone();  // 显式复制

    println!("{}", s1);  // OK
    println!("{}", s2);  // OK
}
```

### 拷贝（Copy）vs 移动（Move）

某些类型实现了`Copy` trait，它们会拷贝而不是移动：

```rust
fn main() {
    let x = 5;          // i32实现了Copy
    let y = x;          // 拷贝，不是移动

    println!("{}", x);  // OK
    println!("{}", y);  // OK
}
```

实现了`Copy`的类型：
- 所有整数类型（如u32, i64等）
- 布尔类型（bool）
- 浮点数类型（f64, f32）
- 字符类型（char）
- 元组（当且仅当其元素都是Copy类型）

## 借用（Borrowing）

### 不可变借用

```rust
fn calculate_length(s: &String) -> usize {  // 借用，不获取所有权
    s.len()
}  // s离开作用域，但因为它只是引用，所以不会销毁数据

fn main() {
    let s1 = String::from("Hello");
    let len = calculate_length(&s1);  // 传递引用

    println!("'{}'的长度是{}", s1, len);  // s1仍然有效
}
```

### 可变借用

```rust
fn change(some_string: &mut String) {
    some_string.push_str(", world");
}

fn main() {
    let mut s = String::from("Hello");
    change(&mut s);
    println!("{}", s);  // Hello, world
}
```

### 借用规则

1. **同一时间，可以有多个不可变借用**
2. **但只能有一个可变借用**
3. **可变借用和不可变借用不能同时存在**

```rust
fn borrowing_rules() {
    let mut s = String::from("hello");

    let r1 = &s;        // OK
    let r2 = &s;        // OK
    // let r3 = &mut s;  // 错误！不能同时有可变和不可变借用
    println!("{} and {}", r1, r2);

    let r3 = &mut s;    // OK，r1和r2已经不再使用
    println!("{}", r3);
}
```

## 生命周期（Lifetimes）

生命周期是Rust用来防止悬垂引用的机制。

### 悬垂引用的例子

```rust
fn main() {
    let r;                // 声明变量r

    {                     // 新的作用域
        let x = 5;        // 变量x
        r = &x;           // 错误！x的生命周期不够长
    }                     // x离开作用域，被销毁

    println!("r: {}", r); // 使用已经销毁的x
}
```

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

    let result = longest(&string1, string2);
    println!("最长的字符串是: {}", result);
}
```

### 生命周期省略

在某些情况下，Rust可以自动推断生命周期：

```rust
// 这两个函数是等价的
fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();
    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return &s[0..i];
        }
    }
    &s[..]
}

// 显式标注版本
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

## 实战例子：链表实现

```rust
// 链表节点
struct Node<T> {
    data: T,
    next: Option<Box<Node<T>>>,  // 使用Box来处理递归类型
}

// 链表
struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
}

impl<T> LinkedList<T> {
    fn new() -> Self {
        LinkedList { head: None }
    }

    fn push(&mut self, data: T) {
        let new_node = Node {
            data,
            next: self.head.take(),  // 取出当前head
        };
        self.head = Some(Box::new(new_node));
    }

    fn pop(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            node.data
        })
    }
}

fn main() {
    let mut list = LinkedList::new();

    list.push(1);
    list.push(2);
    list.push(3);

    while let Some(data) = list.pop() {
        println!("{}", data);
    }
}
```

## 深入理解：Rust vs C++内存管理

### C++的智能指针对比

```cpp
#include <memory>
#include <vector>

void cpp_smart_pointers() {
    // unique_ptr - 类似于Rust的Box
    auto unique = std::make_unique<int>(42);

    // shared_ptr - 引用计数
    auto shared1 = std::make_shared<int>(42);
    auto shared2 = shared1;  // 两个shared_ptr共享所有权

    // weak_ptr - 解决循环引用
    std::weak_ptr<int> weak = shared1;
}
```

### Rust的等价实现

```rust
fn rust_smart_pointers() {
    // Box - 独占所有权
    let boxed = Box::new(42);

    // Rc - 引用计数
    let shared1 = std::rc::Rc::new(42);
    let shared2 = std::rc::Rc::clone(&shared1);

    // Weak - 解决循环引用
    let weak = std::rc::Rc::downgrade(&shared1);
}
```

## 总结

今天我们学习了：
- **所有权三规则**：Rust内存管理的基础
- **移动语义**：避免不必要的复制
- **借用系统**：灵活的引用机制
- **生命周期**：防止悬垂引用
- **智能指针**：Box、Rc、Arc等

**关键点：**
- Rust在编译时就保证了内存安全
- 没有运行时开销，零成本抽象
- 通过所有权系统实现了"无GC的内存安全"

**明天预告：** 模式匹配和Rust强大的控制流！

## 练习作业

1. 实现一个简单的文本编辑器，支持插入、删除、显示文本
2. 编写一个函数，返回字符串中最长的单词
3. 实现一个二叉搜索树，深入理解所有权和生命周期

---

*如果你是C++开发者，重点关注Rust如何通过编译时检查解决了C++的内存安全问题；如果你是Python开发者，重点关注Rust如何在保证内存安全的同时，避免了GC的运行时开销。*