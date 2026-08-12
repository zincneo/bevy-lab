# Lab 006：使用 `bsn!` 声明静态 UI

本实验把 Lab 005 的控件画廊改写为 `bsn!` 场景声明。按钮、复选框、单选组、滑动
条、文本输入框、面板和父子关系都写在独立的场景函数中；交互 Observer、业务
Resource 和每帧状态同步仍然是普通 Rust 函数。

## 运行示例

```bash
nix develop
just run ui 006
```

## 一、`bsn!` 是什么

`bsn!` 是 Bevy Scene Notation（Bevy 场景表示法）的宏。它把一段更接近“实体树”的
声明转换为 Bevy 的 `Scene`：场景描述应该创建哪些 Entity、每个 Entity 添加哪些
Component，以及这些 Entity 之间有什么关系。

最小示例：

```rust
fn panel() -> impl Scene {
    bsn! {
        Node { width: px(320), height: px(80) }
        BackgroundColor(Color::srgb(0.1, 0.1, 0.1))
        Children [
            Text::new("A static child")
        ]
    }
}
```

这个场景仍然要通过 `spawn_scene` 或相关的场景扩展方法加入 World：

```rust
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn_scene(panel());
}
```

`bsn!` 不是另一套 UI 框架，也不是 HTML。它只是把原来嵌套的
`commands.spawn(...).add_children(...)` 写法变成了静态场景声明。

## 二、BSN 中最常用的语法

### 1. 空白分隔的内容属于同一个 Entity

```rust
bsn! {
    Node { width: px(200) }
    BackgroundColor(Color::BLACK)
    Button
}
```

上面三个条目会添加到同一个 Entity。没有显式值的 Component 使用它的
`Default`，带括号或字段的写法用于提供初始值：

```rust
SliderValue(50.0)
SliderRange::new(0.0, 100.0)
TabIndex(0)
```

### 2. `Children [...]` 声明父子关系

方括号中的每个逗号分隔项都是一个子场景：

```rust
Node { flex_direction: FlexDirection::Column }
Children [
    Text::new("Title"),
    (
        Node { width: px(200) }
        BackgroundColor(Color::srgb(0.2, 0.2, 0.2))
    ),
]
```

这会创建一个根 Entity 和两个子 Entity。子场景还可以继续拥有自己的
`Children [...]`，因此可以直接表达 UI 的树状结构。

### 3. 使用函数组合静态场景

`bsn!` 场景可以嵌套普通函数返回的 `impl Scene`：

```rust
fn demo_root() -> impl Scene {
    bsn! {
        Node
        Children [button_panel(), slider_panel(), text_input_panel()]
    }
}
```

本实验使用 `panel()`、`section_title()` 和各个控件面板函数，把一个很长的 UI 树
拆成多个可阅读、可复用的静态片段。它们仍然在启动时组合成同一棵 Entity 树。

### 4. 在场景中绑定 Observer

`on(system)` 可以把 Entity Observer 直接附加到场景中的实体：

```rust
(
    Button
    on(button_activated)
    Node { width: px(440), height: px(48) }
)
```

这和 `commands.entity(entity).observe(button_activated)` 是同一种 Observer，只是
Entity 还没有手动创建时，就可以在 BSN 声明中完成绑定。

## 三、用 BSN 重写 Lab 005

### 1. 静态树放在 `demo_root`

006 的启动系统只负责创建相机和场景：

```rust
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn_scene(demo_root());
}
```

`demo_root()` 中声明根节点、滚动区域、标题以及五个面板：

```rust
fn demo_root() -> impl Scene {
    bsn! {
        ScrollArea
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            overflow: Overflow::scroll_y(),
        }
        TabGroup
        Children [
            button_panel(),
            checkbox_panel(),
            radio_panel(),
            slider_panel(),
            text_input_panel(),
        ]
    }
}
```

这部分只描述“有哪些实体、它们如何嵌套、初始组件是什么”，不负责每一帧改变
颜色、文字或 Resource。

### 2. 动态行为仍然使用普通 Rust 系统

静态声明可以直接附加 Observer，但复杂状态同步仍然写成普通函数：

- `button_activated` 修改 `WidgetState::button_activations`；
- `checkbox_changed` 保存 `ValueChange<bool>`；
- `radio_changed` 把选中 Entity 转换为 `RadioChoice`；
- `slider_value_changed` 同时更新 `SliderValue` 和百分值 Resource；
- `commit_text_on_enter` 检查当前 `InputFocus`，只在 Enter 时提交文本；
- `update_*` 系统根据组件和 Resource 修改视觉状态。

例如滑动条 Observer 仍然是普通 Rust 函数，只是 Observer 的注册位置从启动代码移到
BSN 场景中：

```rust
fn slider_value_changed(
    event: On<ValueChange<f32>>,
    mut state: ResMut<WidgetState>,
    mut commands: Commands,
) {
    state.slider_percent = event.value;
    commands
        .entity(event.source)
        .insert(SliderValue(event.value));
}
```

## 四、BSN 与手动 `Commands` 的对应关系

| 手动写法 | BSN 写法 |
| --- | --- |
| `commands.spawn((Node { .. }, Button))` | `bsn! { Node { .. } Button }` |
| `commands.entity(parent).add_child(child)` | `Children [child_scene()]` |
| `commands.entity(entity).observe(handler)` | `on(handler)` 放在该 Entity 的场景中 |
| 多次 `spawn` 后再保存 Entity | 用场景中的组件标记查询，例如 `With<DemoSlider>` |
| 启动系统中逐个构建 UI | `commands.spawn_scene(demo_root())` |

BSN 只改变“静态实体结构的书写方式”，不改变 ECS 的 Entity、Component、Children、
Observer 或 System。理解 005 中的 ECS UI 组成后，可以把静态部分迁移到 BSN，而不
需要重新学习交互逻辑。

## 五、什么时候使用 BSN

适合放入 `bsn!` 的内容：

- 固定的 UI 层级和父子关系；
- 初始布局、颜色、字体和控件组件；
- 固定的 Observer 注册；
- 可拆分、可复用的静态场景函数。

仍然应该使用普通 System 或 Commands 的内容：

- 根据游戏状态动态创建或删除实体；
- 每帧修改布局、颜色、文字和控件状态；
- 读取输入并更新 Resource；
- 需要运行时决定的子节点数量。

因此 006 的结构是：`bsn!` 负责静态 UI 场景，ECS System 负责运行中的数据和行为。
