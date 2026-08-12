# Lab 009：静态 UI 与动态 UI

本实验介绍 Bevy UI 中静态 UI 和动态 UI 的区别，并演示如何根据用户在输入框中输入
的数量动态创建按钮。输入 `0` 到 `12` 之间的数字后按 Enter，下面的面板会销毁
旧按钮并创建对应数量的新按钮。

## 运行示例

```bash
nix develop
just run ui 009
```

## 一、静态 UI 是什么

静态 UI 指的是 UI 的实体结构在编写代码时就已经确定。例如一个固定的设置面板：

```text
RootPanel
├── Title
├── VolumeSlider
├── FullscreenCheckbox
└── ApplyButton
```

可以使用普通的 `commands.spawn`、`children![]` 或 `bsn!` 创建它。静态并不代表它
永远不能变化：滑块的值、按钮的文字、颜色和 `Visibility` 仍然可以由系统修改，只是
实体的数量和父子关系通常不会随数据变化。

静态 UI 适合：

- 主菜单、设置页面和固定的 HUD；
- 控件数量已知的表单；
- 只需要更新组件值而不需要重建结构的页面。

## 二、动态 UI 是什么

动态 UI 指的是 UI 的实体结构由运行时数据决定。数据变化时，系统需要创建、销毁或
重新组织 Entity，而不仅仅是修改已有组件。例如：

```text
配置文件或用户输入：button_count = 3
                    ↓
系统创建 3 个按钮 Entity
                    ↓
按钮挂到 ButtonPanel 的 Children
```

动态数据可以来自：

- 用户在 `TextInput` 中输入的值；
- 本地配置文件、存档或服务器返回的数据；
- ECS 中的实体列表，例如物品、任务、排行榜和聊天消息；
- 窗口尺寸、语言或权限变化导致的 UI 结构变化。

动态 UI 的关键不是“动态修改一个 `Text`”，而是让系统根据数据决定要存在多少个
Entity、每个 Entity 具有什么组件以及它们如何建立父子关系。

## 三、示例的运行流程

示例中的静态部分在 `Startup` 中创建一次：

```text
Root
├── 标题
├── 说明文字
├── TextInput
├── 状态文字
└── ButtonPanel
```

`ButtonPanel` 本身是固定的，但它的子按钮是动态内容。按下 Enter 后，系统执行：

1. 检查当前焦点是否在输入框上；
2. 读取 `EditableText` 的字符串；
3. 将字符串解析为数量并限制在 `0..=12`；
4. 使用 `despawn_children()` 移除旧按钮；
5. 使用 `Commands` 创建新按钮，并通过 `ChildOf(panel)` 加入面板；
6. 给每个按钮添加 `DynamicButton(index)`，让 Observer 知道点击的是哪个按钮。

代码的核心结构如下：

```rust
commands.entity(panel).despawn_children();

for index in 0..count {
    let button = commands.spawn((
        Button,
        DynamicButton(index),
        ChildOf(panel),
        Node { ..default() },
        Text::new(format!("Button {index}")),
    )).id();

    commands.entity(button).observe(on_button_activated);
}
```

`Commands` 的修改会在当前系统结束后应用到 World。下一次 UI 布局阶段执行时，Bevy
会根据新的 `Children` 关系重新计算面板布局。

## 四、输入值与动态结构的关系

输入框的内容本身只是数据，系统决定如何使用它：

```text
EditableText.value()
        ↓ parse
button_count
        ↓ compare with previous value
only when changed
        ↓
despawn old children + spawn new children
```

示例只在按下 Enter 且数量真正变化时重建按钮，不会在每一帧都销毁和创建实体。这样
可以避免不必要的 ECS 结构变化，也保留了用户正在编辑输入框时的状态。

输入内容无效时，示例把数量当作 `0`；过大的数量会被限制为 `12`。实际项目通常还
应该显示校验错误，而不是静默使用默认值。

## 五、从本地配置生成 UI

从本地配置文件生成 UI 的流程和输入框完全相同，区别只在于数据来源：

```text
AssetServer / 配置资源
        ↓
读取 MenuConfig、ItemConfig 等数据
        ↓
系统遍历配置项
        ↓
为每一项创建 Entity
        ↓
通过 ChildOf 挂到固定的父面板
```

伪代码可以写成：

```rust
struct MenuConfig {
    buttons: Vec<String>,
}

fn build_menu(config: &MenuConfig, panel: Entity, mut commands: Commands) {
    commands.entity(panel).despawn_children();

    for label in &config.buttons {
        commands.spawn((
            Button,
            ChildOf(panel),
            Text::new(label.clone()),
        ));
    }
}
```

配置文件加载完成后调用这个系统或发送一个消息即可。配置文件中的字符串、数量和
条目列表应该先经过校验，再用于创建 UI；不要让未经限制的外部数据直接创建无限多
的实体。

## 六、重建还是更新已有实体

数据变化不一定要重建 UI，需要先判断变化的是“值”还是“结构”：

| 数据变化 | 推荐方式 |
| --- | --- |
| 按钮文字、颜色、进度值变化 | 查询已有 Entity，修改 `Text`、`BackgroundColor` 或其它组件。 |
| 按钮数量变化 | 销毁或复用动态子实体，然后重新建立父子关系。 |
| 列表内容变化但数量相近 | 可以只更新已有行，也可以在学习阶段先整体重建。 |
| 页面完全切换 | 销毁旧页面根 Entity 的子树，再生成新页面。 |

学习阶段使用“销毁旧子实体再重建”的方式最容易理解。实际项目中如果列表很大、
更新频繁，可以进一步学习实体复用、稳定的业务 ID、局部更新和虚拟列表。

## 七、动态 UI 中组件的作用

动态创建的实体同样需要通过组件表达用途：

- `Button`：让 Bevy UI Widgets 把实体当作按钮处理；
- `DynamicButton(usize)`：保存应用自己的按钮编号；
- `ChildOf(panel)`：把按钮加入面板的 UI 树；
- `Node`、`BackgroundColor`、`Text`：分别描述布局、视觉和内容；
- `observe(...)`：为新创建的按钮注册点击后的行为。

因此，动态 UI 并不是脱离 ECS 的特殊机制，而是运行时继续使用同样的 Entity、
Component、Children、Commands 和 Observer，只是创建这些内容的时机从 `Startup`
变成了数据变化之后。
