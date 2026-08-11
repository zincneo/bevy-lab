# Lab 009：Message

## 学习目标

`Message` 是 Bevy ECS 中按类型保存的缓冲消息，适合让一个 System 发送数据、另一个或多个 System 在之后读取。它不是直接调用目标函数，而是先写入 `World` 中的消息队列，再由读取方在自己的 System 中轮询。

## Message 是什么

消息类型通常是一个普通结构体，并通过 `#[derive(Message)]` 实现 `Message`：

```rust
#[derive(Message, Debug)]
struct ScoreChanged {
    value: u32,
}
```

`Message` 类型必须满足 `Send + Sync + 'static`。派生宏会在类型不满足要求时给出编译错误；由普通数值、字符串和其他线程安全字段组成的结构体通常可以直接派生。

每个消息类型在 `World` 中对应一个 `Messages<T>` Resource：

```text
World
└── Messages<ScoreChanged>
    ├── 本次更新写入的消息
    └── 上次更新仍可读取的消息
```

同一类型的消息共享一个队列，但每个 `MessageReader<T>` 都有自己的读取游标，因此多个读取 System 可以各自收到同一条消息。

## 注册消息类型

在使用 `MessageWriter<T>` 或 `MessageReader<T>` 前，先注册消息类型：

```rust
App::new()
    .add_message::<ScoreChanged>()
    .run();
```

`add_message::<T>()` 会初始化 `Messages<T>` Resource，并把消息更新系统加入应用的基础流程。通常不需要手动创建 `Messages<T>`，也不需要自己调用 `Messages::update()`。

同一个消息类型重复调用 `add_message` 不会重复注册。一个 App 可以注册任意多个互相独立的消息类型：

```rust
app.add_message::<ScoreChanged>()
    .add_message::<PlayerJoined>();
```

## 发送消息

### `MessageWriter<T>` 发送单条消息

System 使用 `MessageWriter<T>` 写入消息，参数需要声明为 `mut`：

```rust
fn report_score(mut writer: MessageWriter<ScoreChanged>) {
    writer.write(ScoreChanged { value: 100 });
}
```

`write` 只是把消息放入当前消息缓冲区，不会立即调用某个读取 System。消息会在后续读取 System 运行时被处理。

### 批量发送

需要发送多条同类型消息时，可以使用 `write_batch`：

```rust
fn report_scores(mut writer: MessageWriter<ScoreChanged>) {
    writer.write_batch([
        ScoreChanged { value: 10 },
        ScoreChanged { value: 20 },
        ScoreChanged { value: 30 },
    ]);
}
```

`write_batch` 表达批量写入，也比连续调用多次 `write` 更适合一次产生大量消息。

### 发送默认消息

空消息或实现 `Default` 的消息可以使用 `write_default`：

```rust
#[derive(Message, Default)]
struct RefreshRequested;

fn request_refresh(mut writer: MessageWriter<RefreshRequested>) {
    writer.write_default();
}
```

## 读取消息

### `MessageReader<T>`

System 使用 `MessageReader<T>` 读取自己还没有读过的消息：

```rust
fn apply_score_changes(mut reader: MessageReader<ScoreChanged>) {
    for message in reader.read() {
        println!("新的分数：{}", message.value);
    }
}
```

`MessageReader` 的参数必须声明为 `mut`，因为 `read()` 会推进这个 Reader 自己的游标。再次调用 `read()` 时，不会重复返回已经读过的消息。

每个 Reader 都独立记录游标：

```rust
fn update_ui(mut reader: MessageReader<ScoreChanged>) {
    for message in reader.read() {
        println!("UI 收到：{}", message.value);
    }
}

fn write_log(mut reader: MessageReader<ScoreChanged>) {
    for message in reader.read() {
        println!("日志收到：{}", message.value);
    }
}
```

同一条 `ScoreChanged` 会分别被这两个 System 读取一次，不会因为 UI Reader 先读取而让日志 Reader 丢失。

### Reader 的常用方法

| 方法 | 作用 |
| --- | --- |
| `read()` | 返回当前 Reader 尚未读取的消息，并推进游标。 |
| `read_with_id()` | 读取消息，同时返回每条消息的 `MessageId`。 |
| `len()` | 查看当前 Reader 尚未读取的消息数量，不推进游标。 |
| `is_empty()` | 判断当前 Reader 是否没有可读取消息。 |
| `clear()` | 直接跳过当前 Reader 尚未读取的消息。 |

大多数 System 只需要 `for message in reader.read()`。需要先判断是否有消息但不立即遍历时，可以使用 `is_empty()` 或 `len()`。

### `PopulatedMessageReader<T>`

如果没有消息时整个 System 都不需要运行，可以使用 `PopulatedMessageReader<T>`：

```rust
fn play_sound(mut reader: PopulatedMessageReader<RefreshRequested>) {
    for _message in reader.read() {
        println!("有刷新请求，执行一次处理");
    }
}
```

消息为空时，Bevy 会跳过这个 System；有至少一条消息时才会进入函数。若 System 即使没有消息也需要做其他工作，则使用普通的 `MessageReader<T>`。

## 消息何时可以被读取

消息是缓冲数据，不是即时回调。典型流程是：

```text
发送 System
    ↓
MessageWriter<T>::write
    ↓
消息进入 Messages<T> 缓冲区
    ↓
读取 System
    ↓
MessageReader<T>::read
```

在同一 Schedule 中，如果发送方和读取方有明确的先后关系，读取方可以在本次 Schedule 中读到新消息：

```rust
app.add_systems(Update, (report_score, apply_score_changes).chain());
```

如果没有安排顺序，两个 System 可能并行执行，读取方可能要到下一次主循环才读到消息。因此需要依赖消息立即被处理时，应使用 `.chain()`、`.before()` 或 `.after()` 建立明确顺序。

消息更新系统通常由 `add_message` 自动注册到基础流程中。消息会在一个更新边界后继续保留一段时间，让发送方和读取方不必紧挨着执行；如果 Reader 长时间不运行，旧消息最终会被自动清理。需要可靠处理消息时，应让对应 Reader 至少每次主循环运行一次。

## 只在有消息时运行 System

除了 `PopulatedMessageReader`，还可以用 `on_message::<T>` 作为运行条件：

```rust
fn process_refresh() {
    println!("收到至少一条 RefreshRequested");
}

app.add_systems(
    Update,
    process_refresh.run_if(on_message::<RefreshRequested>),
);
```

`on_message` 会使用自己的 Reader 检查是否有新消息。条件成立时，目标 System 仍可以使用另一个 `MessageReader<T>` 读取这些消息；不同 Reader 的游标彼此独立。

## Message 与 Resource 的关系

`Messages<T>` 本身是 Resource，但业务代码一般不直接操作它，而是使用：

- `MessageWriter<T>`：写入消息；
- `MessageReader<T>`：读取消息；
- `PopulatedMessageReader<T>` 或 `on_message::<T>`：没有消息时跳过处理。

只有在测试、工具代码或需要直接控制缓冲区生命周期时，才考虑通过 `World` 访问 `Messages<T>`。正常 System 使用 Writer 和 Reader 可以让消息游标、调度访问和自动清理由 Bevy 管理。

## 一个最小的使用流程

```text
定义 #[derive(Message)] 的消息类型
        ↓
App::add_message::<T>() 注册消息队列
        ↓
发送 System 使用 MessageWriter<T>::write
        ↓
读取 System 使用 MessageReader<T>::read
        ↓
需要时使用 PopulatedMessageReader 或 on_message 限制运行
        ↓
基础流程自动更新和清理消息缓冲区
```

选择 API 时可以遵循下面的规则：

1. 发送单条消息用 `write`，一次发送多条用 `write_batch`，空消息用 `write_default`。
2. 普通读取用 `MessageReader<T>`；没有消息时希望跳过 System，用 `PopulatedMessageReader<T>`。
3. 只需要判断“有没有新消息”时，用 `run_if(on_message::<T>)`。
4. 发送和读取必须依赖同一帧内的先后关系时，用 `.chain()`、`.before()` 或 `.after()` 明确安排顺序。
5. Reader 应定期运行，否则消息在缓冲区生命周期结束后会被自动丢弃。
