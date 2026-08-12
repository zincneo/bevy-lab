# Lab 004：UI 交互事件

UI 节点除了显示内容，还需要知道用户是否把指针移到节点上、是否按下鼠标、是否
完成点击，以及当前是否有键盘输入。本实验使用当前 Bevy `main` 的 UI picking 和
UI Widgets API，演示这些最常见的交互入口。

## 运行示例

```bash
nix develop
just run ui 004
```

窗口中有三个交互区域：

- `Pointer events`：把鼠标移入、移出、按下、释放或点击，观察面板文字和颜色变化；
- `Press Enter or Space`：一个 Bevy `Button`，鼠标点击或键盘激活后会修改按钮文字；
- `Global shortcut`：按 `K` 或 `Escape`，由普通系统读取全局键盘输入并更新文字。

## 一、UI 指针事件

当前 Bevy 使用 picking 产生通用的 `Pointer<E>` 事件。UI 的 picking 后端已经由
`DefaultPlugins` 提供，因此这个示例不需要额外添加 picking 插件。

常用事件如下：

| 事件 | 触发时机 |
| --- | --- |
| `Pointer<Over>` | 指针进入目标或其可交互区域。 |
| `Pointer<Out>` | 指针离开目标。 |
| `Pointer<Press>` | 指针按钮在目标上按下。 |
| `Pointer<Release>` | 指针按钮在目标上释放。 |
| `Pointer<Click>` | 同一个目标先收到按下，再收到释放，形成一次点击。 |

`Pointer<E>` 包含目标 Entity、指针 ID、位置以及具体事件数据。例如
`Pointer<Press>` 的事件数据中有 `button`，可以区分主键、次键等指针按钮。

### 使用 Observer 接收事件

指针事件是 Entity Event，最直接的用法是把 Observer 挂到目标 Entity：

```rust
commands
    .entity(pointer_panel)
    .observe(pointer_over)
    .observe(pointer_out)
    .observe(pointer_press)
    .observe(pointer_release)
    .observe(pointer_click);
```

对应的 Observer 系统只需要把 `On<Pointer<E>>` 写成第一个参数：

```rust
fn pointer_click(
    event: On<Pointer<Click>>,
    mut statuses: Query<&mut Text, With<PointerStatus>>,
) {
    info!("Pointer clicked {:?}", event.entity);
    if let Ok(mut status) = statuses.single_mut() {
        status.0 = "Clicked".to_string();
    }
}
```

Observer 只会在对应事件发生时运行，不需要在 `Update` 中每帧轮询鼠标。指针事件
默认可以沿 `ChildOf` 关系向父实体传播；如果只希望当前 Entity 处理事件，可以在
Observer 中调用 `event.propagate(false)`。

### 与 Basic Observer 的关系

这里的 Observer 和 Basic Lab 015 介绍的是同一个 ECS 机制，区别只在于监听的事件
来源和注册范围：

| 写法 | 含义 |
| --- | --- |
| `App::add_observer(system)` | 注册全局 Observer，匹配的事件触发时可以从整个 World 接收它。 |
| `commands.entity(entity).observe(system)` | 把 Observer 绑定到一个 Entity，只处理发送给该 Entity 或沿层级传播到该 Entity 的事件。 |
| `On<Greeting>` | Basic 示例中的自定义事件。 |
| `On<Pointer<Click>>` | UI picking 产生的指针点击事件。 |

两种 Observer 都是“事件触发时才运行的 System”，也都使用 `On<EventType>` 读取事
件。UI 只是把事件来源换成了 picking 插件，并利用 Entity 绑定和父子关系传播来定位
具体的界面元素。

本实验中的 `Hovered` 是可查询的状态 Component，`keyboard_shortcut` 是注册到
`Update` 的普通 System；它们不是 Observer。只有 `pointer_*` 和
`button_activated` 这些接收 `On<...>` 的函数属于 Observer。

### `Over`、`Out` 与悬浮状态

如果只关心“进入”和“离开”这两个瞬间，使用 `Pointer<Over>` 和 `Pointer<Out>`。
如果需要在普通系统中持续查询当前是否悬浮，可以给 Entity 添加：

```rust
Hovered::default()
```

之后查询 `&Hovered`：

```rust
fn read_hover_state(query: Query<&Hovered>) {
    for hovered in &query {
        if hovered.get() {
            // 当前正在悬浮
        }
    }
}
```

`Hovered` 是 picking 系统维护的状态组件；它和 `Pointer<Over>` 的区别是，前者表
示当前状态，后者表示一次进入动作。

## 二、按下、释放和点击

`Press`、`Release` 和 `Click` 是三个不同的阶段：

```text
按下鼠标       Pointer<Press>
松开鼠标       Pointer<Release>
按下和释放目标 Pointer<Click>
```

`Click` 只有在按下和释放的目标满足点击条件时才会产生。如果按下后把指针移到其
它节点再释放，通常不会对原节点产生 `Click`。

示例在不同 Observer 中修改了面板颜色：按下时显示按下颜色，释放后显示悬浮颜色，
点击时更新文字。这样可以直接看到三个阶段并不是同一个事件。

## 三、使用 `Button` 处理控件交互

当前 Bevy 的 `ui_widgets::Button` 是一个无样式的行为组件。它负责维护按钮的按下
状态，并把一次完整的鼠标点击转换成 `Activate` Entity Event：

```rust
commands.entity(button_entity).observe(
    |_activate: On<Activate>, mut labels: Query<&mut Text, With<KeyboardButtonLabel>>| {
        if let Ok(mut label) = labels.single_mut() {
            label.0 = "Activated".to_string();
        }
    },
);
```

按钮的颜色、边框和文字仍然需要自己添加。`Button` 只是交互行为，不是完整的视觉
控件。

按钮还会在被键盘聚焦时，把 `Enter` 或 `Space` 转换成同一个 `Activate` 事件。示例
使用 `AutoFocus` 让这个按钮启动时获得输入焦点，因此可以直接按 `Enter` 或 `Space`
测试，无需先实现 Tab 导航。

当前 `main` 中仍然可以看到旧的 `Interaction` 组件，但它已经标记弃用。新的 UI
代码应优先学习 `Pointer<E>`、`Hovered`、`Pressed` 和 `ui_widgets::Button`。

## 四、键盘输入

### 1. 在系统中读取全局按键状态

最常用的方式是读取 `ButtonInput<KeyCode>` Resource：

```rust
fn keyboard_shortcut(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut statuses: Query<&mut Text, With<KeyboardStatus>>,
) {
    if keyboard.just_pressed(KeyCode::KeyK) {
        // K 在这一帧刚刚按下
    }

    if keyboard.pressed(KeyCode::KeyK) {
        // K 当前仍处于按下状态
    }

    if keyboard.just_released(KeyCode::KeyK) {
        // K 在这一帧刚刚释放
    }
}
```

`ButtonInput<KeyCode>` 是全局输入状态，不属于某个 UI Entity。任何系统都可以读取
它，所以适合实现全局快捷键、游戏控制和不依赖焦点的 UI 快捷操作。示例使用它监
听 `K` 和 `Escape`。

### 2. 读取原始键盘消息

如果需要每一条按键消息，而不只是查询当前状态，可以读取 `KeyboardInput` Message：

```rust
fn raw_keyboard_messages(mut messages: MessageReader<KeyboardInput>) {
    for message in messages.read() {
        info!("Keyboard input: {:?}", message);
    }
}
```

这适合观察按下/释放事件本身。文字输入还需要处理字符、输入法和焦点，不能只依赖
`KeyCode`；文本输入会在后续专题单独介绍。

### 3. 把键盘消息发送给聚焦的 UI Entity

当应用启用了输入焦点系统时，原始 `KeyboardInput` 会被分发给当前
`InputFocus` 指向的 Entity，UI 控件可以用：

```rust
On<FocusedInput<KeyboardInput>>
```

接收聚焦后的键盘输入。`ui_widgets::Button` 内部就是通过这种机制处理聚焦后的
`Enter` 和 `Space`，然后触发 `Activate`。普通全局快捷键不需要这一层，直接读取
`ButtonInput<KeyCode>` 即可。

## 五、几种输入方式的职责边界

| 需求 | 推荐入口 |
| --- | --- |
| 指针进入或离开一个 UI Entity | `On<Pointer<Over>>`、`On<Pointer<Out>>` |
| 指针按下、释放或点击 | `On<Pointer<Press>>`、`On<Pointer<Release>>`、`On<Pointer<Click>>` |
| 查询当前是否悬浮 | `Hovered` |
| 普通按钮点击和键盘激活 | `ui_widgets::Button` + `On<Activate>` |
| 全局快捷键 | `Res<ButtonInput<KeyCode>>` |
| 接收原始键盘消息 | `MessageReader<KeyboardInput>` |
| 接收当前焦点 Entity 的键盘消息 | `On<FocusedInput<KeyboardInput>>` |

本实验只演示事件接收和最基本的状态反馈，不涉及拖拽、滚动、输入框、Tab 导航或
复杂的控件状态管理。
