# 第八天：并发和并行——Rust的"无畏并发"

## 什么是并发和并行？

并发（Concurrency）是处理多个任务的能力，而并行（Parallelism）是同时执行多个任务的能力。Rust通过其所有权系统，实现了"无畏并发"（Fearless Concurrency）。

### 从C++的角度理解

**C++的并发：**
```cpp
#include <thread>
#include <mutex>
#include <atomic>

void cpp_concurrency() {
    std::mutex mtx;
    int shared_data = 0;

    std::thread t1([&mtx, &shared_data]() {
        std::lock_guard<std::mutex> lock(mtx);
        shared_data++;
    });

    std::thread t2([&mtx, &shared_data]() {
        std::lock_guard<std::mutex> lock(mtx);
        shared_data++;
    });

    t1.join();
    t2.join();
}
```

### 从Python的角度理解

**Python的并发：**
```python
import threading
import queue

def python_concurrency():
    shared_queue = queue.Queue()

    def producer():
        for i in range(5):
            shared_queue.put(i)

    def consumer():
        while not shared_queue.empty():
            item = shared_queue.get()
            print(f"消费: {item}")

    t1 = threading.Thread(target=producer)
    t2 = threading.Thread(target=consumer)

    t1.start()
    t2.start()

    t1.join()
    t2.join()
```

## Rust中的线程基础

### 创建线程

```rust
use std::thread;
use std::time::Duration;

fn basic_threads() {
    // 主线程
    let handle = thread::spawn(|| {
        println!("子线程开始");
        thread::sleep(Duration::from_millis(100));
        println!("子线程结束");
    });

    println!("主线程继续执行");

    // 等待子线程完成
    handle.join().unwrap();
    println!("所有线程完成");
}
```

### 线程间传递数据

```rust
use std::thread;

fn move_data_between_threads() {
    let v = vec![1, 2, 3];

    let handle = thread::spawn(move || {
        println!("子线程接收到向量: {:?}", v);
        // v的所有权已经移动到子线程
    });

    // 不能再使用v
    // println!("主线程: {:?}", v); // 编译错误！

    handle.join().unwrap();
}
```

### 使用Arc共享数据

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn shared_data_with_arc() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter_clone.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("结果: {}", *counter.lock().unwrap());
}
```

## 消息传递

### 使用通道（Channel）

```rust
use std::sync::mpsc;
use std::thread;

fn message_passing() {
    // 创建通道
    let (sender, receiver) = mpsc::channel();

    let sender_clone = sender.clone();

    // 发送者线程1
    thread::spawn(move || {
        let vals = vec![
            String::from("hi"),
            String::from("from"),
            String::from("the"),
            String::from("thread"),
        ];

        for val in vals {
            sender_clone.send(val).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
    });

    // 发送者线程2
    thread::spawn(move || {
        let vals = vec![
            String::from("more"),
            String::from("messages"),
            String::from("for"),
            String::from("you"),
        ];

        for val in vals {
            sender.send(val).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
    });

    // 接收者
    for received in receiver {
        println!("Got: {}", received);
    }
}
```

### 同步和异步通道

```rust
use std::sync::mpsc;
use std::thread;

fn sync_async_channels() {
    // 同步通道
    let (sync_sender, sync_receiver) = mpsc::sync_channel(1);

    thread::spawn(move || {
        sync_sender.send("同步消息").unwrap();
        println!("同步消息已发送");
    });

    let msg = sync_receiver.recv().unwrap();
    println!("收到同步消息: {}", msg);

    // 异步通道
    let (async_sender, async_receiver) = mpsc::channel();

    thread::spawn(move || {
        async_sender.send("异步消息").unwrap();
        println!("异步消息已发送");
    });

    let msg = async_receiver.recv().unwrap();
    println!("收到异步消息: {}", msg);
}
```

## 原子类型

### 基本原子操作

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

fn atomic_operations() {
    let counter = AtomicUsize::new(0);
    let mut handles = vec![];

    for _ in 0..10 {
        let counter_clone = counter.clone();
        let handle = thread::spawn(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("原子计数器: {}", counter.load(Ordering::SeqCst));
}
```

### 内存序（Memory Ordering）

```rust
use std::sync::atomic::{AtomicBool, Ordering};

fn memory_ordering() {
    let flag = AtomicBool::new(false);
    let data = AtomicUsize::new(0);

    // 写入线程
    thread::spawn(move || {
        data.store(42, Ordering::Release);
        flag.store(true, Ordering::Release);
    });

    // 读取线程
    while !flag.load(Ordering::Acquire) {
        thread::yield_now();
    }

    let value = data.load(Ordering::Acquire);
    println!("数据值: {}", value);
}
```

## 实战例子：生产者-消费者模式

```rust
use std::sync::{Arc, Mutex, Condvar};
use std::thread;

struct BoundedBuffer<T> {
    buffer: Mutex<Vec<T>>,
    capacity: usize,
    not_empty: Condvar,
    not_full: Condvar,
}

impl<T> BoundedBuffer<T> {
    fn new(capacity: usize) -> Self {
        BoundedBuffer {
            buffer: Mutex::new(Vec::new()),
            capacity,
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
        }
    }

    fn put(&self, item: T) {
        let mut buffer = self.buffer.lock().unwrap();
        while buffer.len() == self.capacity {
            buffer = self.not_full.wait(buffer).unwrap();
        }
        buffer.push(item);
        self.not_empty.notify_one();
    }

    fn get(&self) -> T {
        let mut buffer = self.buffer.lock().unwrap();
        while buffer.is_empty() {
            buffer = self.not_empty.wait(buffer).unwrap();
        }
        let item = buffer.remove(0);
        self.not_full.notify_one();
        item
    }
}

fn producer_consumer_example() {
    let buffer = Arc::new(BoundedBuffer::new(5));
    let mut handles = vec![];

    // 生产者
    for i in 0..3 {
        let buffer_clone = Arc::clone(&buffer);
        let handle = thread::spawn(move || {
            for j in 0..10 {
                let item = format!("生产者{}-{}", i, j);
                buffer_clone.put(item);
                println!("生产者{} 生产了 {}", i, j);
                thread::sleep(Duration::from_millis(50));
            }
        });
        handles.push(handle);
    }

    // 消费者
    for i in 0..2 {
        let buffer_clone = Arc::clone(&buffer);
        let handle = thread::spawn(move || {
            for _ in 0..15 {
                let item = buffer_clone.get();
                println!("消费者{} 消费了 {}", i, item);
                thread::sleep(Duration::from_millis(100));
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
```

## 实战例子：工作窃取线程池

```rust
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::collections::VecDeque;

struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl ThreadPool {
    fn new(size: usize) -> Self {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("发送终止消息给所有工作线程");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            println!("关闭工作线程 {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    println!("工作线程 {} 执行任务", id);
                    job();
                }
                Message::Terminate => {
                    println!("工作线程 {} 终止", id);
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

fn use_thread_pool() {
    let pool = ThreadPool::new(4);

    for i in 0..8 {
        pool.execute(move || {
            println!("任务 {} 开始", i);
            thread::sleep(Duration::from_millis(100));
            println!("任务 {} 完成", i);
        });
    }

    thread::sleep(Duration::from_millis(500));
} // ThreadPool在这里被drop
```

## 并发模式

### 1. Map-Reduce模式

```rust
use std::thread;

fn map_reduce() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let chunk_size = 3;
    let mut handles = vec![];

    // Map阶段
    for chunk in data.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let handle = thread::spawn(move || {
            chunk.into_iter().map(|x| x * x).collect::<Vec<_>>()
        });
        handles.push(handle);
    }

    // Reduce阶段
    let mut results = Vec::new();
    for handle in handles {
        let chunk_result = handle.join().unwrap();
        results.extend(chunk_result);
    }

    let final_result: i32 = results.iter().sum();
    println!("Map-Reduce结果: {}", final_result);
}
```

### 2. Actor模式

```rust
use std::sync::mpsc;
use std::thread;

struct Actor {
    id: usize,
    receiver: mpsc::Receiver<Message>,
    peers: Vec<mpsc::Sender<Message>>,
}

enum Message {
    Text(String),
    Stop,
}

impl Actor {
    fn new(id: usize, receiver: mpsc::Receiver<Message>) -> Self {
        Actor {
            id,
            receiver,
            peers: Vec::new(),
        }
    }

    fn add_peer(&mut self, peer: mpsc::Sender<Message>) {
        self.peers.push(peer);
    }

    fn run(&mut self) {
        while let Ok(msg) = self.receiver.recv() {
            match msg {
                Message::Text(text) => {
                    println!("Actor {} 收到消息: {}", self.id, text);
                    // 转发给其他peers
                    for peer in &self.peers {
                        let _ = peer.send(Message::Text(format!("转发: {}", text)));
                    }
                }
                Message::Stop => {
                    println!("Actor {} 停止", self.id);
                    break;
                }
            }
        }
    }
}

fn actor_system() {
    let mut senders = Vec::new();
    let mut handles = Vec::new();

    // 创建3个actors
    for i in 0..3 {
        let (sender, receiver) = mpsc::channel();
        senders.push(sender);

        let handle = thread::spawn(move || {
            let mut actor = Actor::new(i, receiver);
            actor.run();
        });
        handles.push(handle);
    }

    // 设置peers
    for i in 0..3 {
        // 这里应该将其他senders添加到每个actor的peers列表中
        // 为简化示例，跳过这部分
    }

    // 发送消息
    senders[0].send(Message::Text("Hello from actor 0".to_string())).unwrap();

    // 等待一段时间
    thread::sleep(Duration::from_millis(100));

    // 停止所有actors
    for sender in senders {
        sender.send(Message::Stop).unwrap();
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
```

## 并发的最佳实践

### 1. 避免数据竞争

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn avoid_data_races() {
    // 好：使用Arc和Mutex
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter_clone.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
```

### 2. 使用原子类型

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

fn use_atomics_when_possible() {
    let counter = AtomicUsize::new(0);

    // 原子操作比互斥锁更轻量级
    counter.fetch_add(1, Ordering::SeqCst);
    println!("计数器值: {}", counter.load(Ordering::SeqCst));
}
```

### 3. 选择合适的并发原语

```rust
fn choose_right_primitives() {
    // 简单的数据共享：Arc
    let data = Arc::new(vec![1, 2, 3]);

    // 需要修改共享数据：Arc + Mutex
    let shared_data = Arc::new(Mutex::new(0));

    // 简单计数器：原子类型
    let counter = AtomicUsize::new(0);

    // 消息传递：通道
    let (sender, receiver) = std::sync::mpsc::channel();
}
```

## 总结

今天我们学习了：
- **线程基础**：创建和管理线程
- **共享状态**：Arc、Mutex、原子类型
- **消息传递**：通道和通信模式
- **并发模式**：生产者-消费者、工作窃取、Actor
- **最佳实践**：避免数据竞争、选择合适的原语

**关键点：**
- Rust的所有权系统在编译时防止数据竞争
- 消息传递比共享状态更安全
- 原子类型提供了无锁的并发
- 选择合适的并发模式至关重要

**明天预告：** 高级主题——宏、异步编程和生态系统！

## 练习作业

1. 实现一个并行的快速排序算法
2. 创建一个简单的Web服务器，处理多个并发请求
3. 实现一个生产者-消费者系统，使用条件变量进行同步

---

*如果你是C++开发者，重点关注Rust的并发安全性如何通过编译器保证；如果你是Python开发者，重点关注Rust如何在没有GIL的情况下实现真正的并行，以及这带来的性能优势。*