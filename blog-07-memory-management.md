# 第七天：内存管理深入理解——智能指针和内存布局

## Rust内存管理的核心概念

Rust的内存管理系统结合了手动内存管理和垃圾回收的优点，提供了既安全又高效的内存管理方案。

### 从C++的角度理解

**C++的内存管理：**
```cpp
#include <memory>

void cpp_memory_management() {
    // 手动管理
    int* raw_ptr = new int(42);
    delete raw_ptr;  // 必须手动释放

    // 智能指针
    std::unique_ptr<int> unique_ptr = std::make_unique<int>(42);
    std::shared_ptr<int> shared_ptr = std::make_shared<int>(42);
    // 自动释放，但有引用计数开销
}
```

**Rust的内存管理：**
```rust
fn rust_memory_management() {
    // 栈分配
    let x = 42;

    // 堆分配
    let box_ptr = Box::new(42);
    // 自动释放，零开销
}
```

### 从Python的角度理解

**Python的内存管理：**
```python
def python_memory_management():
    # 所有对象都在堆上，由GC管理
    data = [1, 2, 3, 4, 5]
    # GC会在适当的时候回收内存
```

## 栈与堆

### 栈内存

```rust
fn stack_memory() {
    // 所有这些都在栈上
    let x = 5;                    // i32
    let y = 3.14;                 // f64
    let z = "hello";              // &str
    let point = (10, 20);         // 元组

    // 栈内存自动管理，LIFO
    println!("x: {}, y: {}, z: {}, point: {:?}", x, y, z, point);
} // 所有变量自动离开作用域
```

### 堆内存

```rust
fn heap_memory() {
    // Box将数据分配在堆上
    let boxed_int = Box::new(42);
    let boxed_string = Box::new(String::from("Hello, heap!"));

    println!("boxed_int: {}", boxed_int);
    println!("boxed_string: {}", boxed_string);
} // Box自动释放堆内存
```

## 智能指针详解

### Box<T> - 独占所有权

```rust
// 用于递归类型
enum List {
    Cons(i32, Box<List>),
    Nil,
}

// 用于大对象
struct LargeData {
    data: [u64; 1000],
}

fn use_box() {
    let list = List::Cons(1, Box::new(List::Cons(2, Box::new(List::Nil))));

    let large_data = Box::new(LargeData {
        data: [0; 1000],
    });

    // Box自动释放内存
}
```

### Rc<T> - 引用计数

```rust
use std::rc::Rc;

fn use_rc() {
    let original = Rc::new(String::from("共享数据"));

    // 创建多个引用
    let reference1 = Rc::clone(&original);
    let reference2 = Rc::clone(&original);
    let reference3 = Rc::clone(&original);

    println!("引用计数: {}", Rc::strong_count(&original)); // 4

    {
        let reference4 = Rc::clone(&original);
        println!("引用计数: {}", Rc::strong_count(&original)); // 5
    } // reference4离开作用域

    println!("引用计数: {}", Rc::strong_count(&original)); // 4
    println!("数据: {}", original);
}
```

### Arc<T> - 原子引用计数

```rust
use std::sync::Arc;
use std::thread;

fn use_arc() {
    let data = Arc::new(vec![1, 2, 3, 4, 5]);

    let mut handles = vec![];

    for i in 0..3 {
        let data_clone = Arc::clone(&data);
        let handle = thread::spawn(move || {
            println!("线程 {} 看到: {:?}", i, data_clone);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
```

### Weak<T> - 弱引用

```rust
use std::rc::{Rc, Weak};

struct Node<T> {
    data: T,
    next: Option<Rc<Node<T>>>,
    prev: Option<Weak<Node<T>>>,
}

fn use_weak() {
    let node1 = Rc::new(Node {
        data: 1,
        next: None,
        prev: None,
    });

    let node2 = Rc::new(Node {
        data: 2,
        next: None,
        prev: Some(Rc::downgrade(&node1)),
    });

    // 通过弱引用访问
    if let Some(weak_ref) = &node2.prev {
        if let Some(strong_ref) = weak_ref.upgrade() {
            println!("前驱节点的数据: {}", strong_ref.data);
        }
    }
}
```

## Cell和RefCell - 内部可变性

### Cell<T>

```rust
use std::cell::Cell;

struct Counter {
    count: Cell<u32>,
}

impl Counter {
    fn new() -> Self {
        Counter { count: Cell::new(0) }
    }

    fn increment(&self) {
        let current = self.count.get();
        self.count.set(current + 1);
    }

    fn get(&self) -> u32 {
        self.count.get()
    }
}

fn use_cell() {
    let counter = Counter::new();
    counter.increment();
    counter.increment();
    println!("计数: {}", counter.get());
}
```

### RefCell<T>

```rust
use std::cell::RefCell;

struct MessageLog {
    messages: RefCell<Vec<String>>,
}

impl MessageLog {
    fn new() -> Self {
        MessageLog {
            messages: RefCell::new(Vec::new()),
        }
    }

    fn add_message(&self, message: &str) {
        self.messages.borrow_mut().push(message.to_string());
    }

    fn get_messages(&self) -> Vec<String> {
        self.messages.borrow().clone()
    }
}

fn use_refcell() {
    let log = MessageLog::new();
    log.add_message("Hello");
    log.add_message("World");

    for msg in log.get_messages() {
        println!("{}", msg);
    }
}
```

## 内存布局和对齐

### 数据结构在内存中的布局

```rust
#[repr(C)]
struct Example {
    a: u8,      // 1字节
    b: u32,     // 4字节
    c: u16,     // 2字节
}

fn show_memory_layout() {
    println!("Example结构体大小: {}", std::mem::size_of::<Example>());
    println!("对齐要求: {}", std::mem::align_of::<Example>());
}

// 使用repr(packed)减少填充
#[repr(packed)]
struct PackedExample {
    a: u8,
    b: u32,
    c: u16,
}

fn show_packed_layout() {
    println!("PackedExample结构体大小: {}", std::mem::size_of::<PackedExample>());
}
```

### 枚举的内存布局

```rust
enum Number {
    Int(i32),
    Float(f64),
    Both(i32, f64),
}

fn show_enum_layout() {
    println!("Number枚举大小: {}", std::mem::size_of::<Number>());
}
```

## 实战例子：内存池实现

```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::ptr::NonNull;

struct PoolAllocator {
    pool: *mut u8,
    size: usize,
    offset: usize,
}

unsafe impl GlobalAlloc for PoolAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        // 计算对齐后的偏移
        let aligned_offset = (self.offset + align - 1) & !(align - 1);

        if aligned_offset + size > self.size {
            // 池空间不足，回退到系统分配器
            System.alloc(layout)
        } else {
            let ptr = self.pool.add(aligned_offset);
            self.offset = aligned_offset + size;
            ptr
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // 简化的实现：不释放单个分配
        // 在实际应用中，需要更复杂的内存管理策略
    }
}

fn use_pool_allocator() {
    const POOL_SIZE: usize = 1024;
    let mut pool = [0u8; POOL_SIZE];

    // 注意：这只是一个示例，实际使用需要更复杂的安全措施
    println!("池分配器示例（概念性）");
}
```

## 实战例子：简单的垃圾回收器

```rust
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
enum Object {
    Number(i32),
    String(String),
    List(Vec<Rc<Object>>),
}

impl Object {
    fn size(&self) -> usize {
        match self {
            Object::Number(_) => std::mem::size_of::<i32>(),
            Object::String(s) => s.len(),
            Object::List(l) => l.len() * std::mem::size_of::<Rc<Object>>(),
        }
    }
}

struct Gc<T> {
    objects: RefCell<HashMap<usize, (Rc<T>, usize)>>,
    next_id: Cell<usize>,
    threshold: usize,
}

impl<T: std::fmt::Debug> Gc<T> {
    fn new(threshold: usize) -> Self {
        Gc {
            objects: RefCell::new(HashMap::new()),
            next_id: Cell::new(0),
            threshold,
        }
    }

    fn allocate(&self, object: T) -> usize {
        let id = self.next_id.get();
        self.next_id.set(id + 1);

        let rc_object = Rc::new(object);
        let size = std::mem::size_of::<T>();

        self.objects.borrow_mut().insert(id, (rc_object, size));

        if self.objects.borrow().len() > self.threshold {
            self.collect();
        }

        id
    }

    fn collect(&self) {
        println!("开始垃圾回收...");

        let mut objects = self.objects.borrow_mut();
        objects.retain(|_, (rc, _)| {
            // 简单的回收策略：只保留引用计数为1的对象
            Rc::strong_count(rc) == 1
        });

        println!("回收完成，剩余对象: {}", objects.len());
    }

    fn get(&self, id: usize) -> Option<Rc<T>> {
        self.objects.borrow().get(&id).map(|(rc, _)| Rc::clone(rc))
    }

    fn stats(&self) {
        let objects = self.objects.borrow();
        let total_objects = objects.len();
        let total_size: usize = objects.values().map(|(_, size)| *size).sum();

        println!("GC统计: 对象数={}, 总大小={}字节", total_objects, total_size);
    }
}

fn use_gc() {
    let gc = Gc::new(10);

    // 分配一些对象
    let id1 = gc.allocate(Object::Number(42));
    let id2 = gc.allocate(Object::String("Hello".to_string()));
    let id3 = gc.allocate(Object::List(vec![
        gc.get(id1).unwrap(),
        gc.get(id2).unwrap(),
    ]));

    gc.stats();

    // 释放一些引用
    {
        let _obj1 = gc.get(id1);
        let _obj2 = gc.get(id2);
    }

    gc.collect();
    gc.stats();
}
```

## 内存安全最佳实践

### 1. 选择合适的智能指针

```rust
// 使用Box - 独占所有权，无开销
fn use_box_example() {
    let data = Box::new(vec![1, 2, 3, 4, 5]);
    // 单个所有者，性能最优
}

// 使用Rc - 共享所有权，单线程
fn use_rc_example() {
    let data = Rc::new(vec![1, 2, 3, 4, 5]);
    let data1 = Rc::clone(&data);
    let data2 = Rc::clone(&data);
    // 多个引用，但不需要修改
}

// 使用Arc - 共享所有权，多线程
fn use_arc_example() {
    let data = Arc::new(vec![1, 2, 3, 4, 5]);
    let data1 = Arc::clone(&data);
    // 在多线程环境中安全共享
}
```

### 2. 避免内存泄漏

```rust
fn avoid_memory_leaks() {
    // 循环引用会导致内存泄漏
    struct Node {
        next: Option<Rc<Node>>,
        prev: Option<Weak<Node>>,  // 使用Weak打破循环
    }

    // 或者使用Box来避免引用计数
    struct ListNode {
        data: i32,
        next: Option<Box<ListNode>>,
    }
}
```

### 3. 性能考虑

```rust
fn performance_considerations() {
    // 栈分配 - 最快
    let stack_value = 42;

    // Box - 堆分配，但无额外开销
    let boxed_value = Box::new(42);

    // Rc - 引用计数开销
    let rc_value = Rc::new(42);

    // RefCell - 运行时借用检查开销
    let cell_value = RefCell::new(42);
}
```

## 总结

今天我们学习了：
- **栈与堆**：内存分配的基础概念
- **智能指针**：Box、Rc、Arc、Weak
- **内部可变性**：Cell和RefCell
- **内存布局**：数据结构在内存中的排列
- **自定义分配器**：内存池和垃圾回收

**关键点：**
- Rust的内存管理既安全又高效
- 智能指针提供了不同的所有权策略
- 理解内存布局有助于优化性能
- Rust在编译时就保证了内存安全

**明天预告：** 并发和并行——Rust的 fearless concurrency！

## 练习作业

1. 实现一个双向链表，使用Rc和Weak处理循环引用
2. 创建一个简单的内存池分配器
3. 实现一个引用计量的智能指针，理解Rc的内部工作原理

---

*如果你是C++开发者，重点关注Rust的智能指针与C++智能指针的异同；如果你是Python开发者，重点关注Rust如何在没有GC的情况下管理内存，以及这带来的性能优势。*