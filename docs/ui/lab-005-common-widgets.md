# Lab 005：`bevy_ui_widgets` 常用控件总览

本实验把 Bevy 当前 UI Widgets 中最常见的控件组合成一个小型控件画廊：按钮、复选
框、单选组、滑动条和文本输入框。画廊使用纵向 Flex 布局，内容超出窗口时可以纵向
滚动。重点不是制作一套固定的视觉主题，而是了解 Bevy 已经提供了哪些控件行为、
它们如何产生事件，以及应用如何把控件状态保存到自己的 Resource 中。

## 运行示例

```bash
nix develop
just run ui 005
```

窗口中的 UI 文本使用英文，便于在没有额外字体资源时直接运行。可以使用鼠标和键盘
操作控件，也可以使用 `Tab` 在可聚焦控件之间移动。

## 一、`bevy_ui_widgets` 的定位

`bevy_ui_widgets` 是 Bevy 提供的标准 UI 控件行为模块。它把鼠标、键盘、输入焦点、
可访问性和控件状态转换封装起来，但不替应用决定控件的外观和业务数据。

这些控件是 **headless、unstyled** 的：控件实体仍然需要 `Node`、
`BackgroundColor`、`BorderColor`、`TextFont` 和子实体来完成布局与绘制。换句话说，
Widgets 减少的是交互逻辑，不是直接提供一个带固定主题的网页组件库。

`DefaultPlugins` 已经包含 `UiWidgetsPlugins`，所以通常不需要额外添加每一个
`ButtonPlugin`、`SliderPlugin` 或 `TextInputPlugin`。示例额外添加
`TabNavigationPlugin`，只是为了让 `TabIndex` 可以移动焦点。

## 二、当前 Widgets 能力地图

下面是当前 `UiWidgetsPlugins` 注册的主要能力。括号中的事件或组件是应用通常需要
关注的状态入口。

| 能力类别 | 类型 | 作用和常见状态入口 |
| --- | --- | --- |
| 激活控件 | `Button` | 鼠标点击或聚焦后的 Enter/Space 触发 `Activate`；`Hovered` 和 `Pressed` 只用于应用自己的视觉反馈。 |
| 布尔选择 | `Checkbox` | 切换 `Checked`，发出 `ValueChange<bool>`；可用于复选框或开关。 |
| 互斥选择 | `RadioGroup`、`RadioButton` | 在一组子选项中选择，组发出包含选中 Entity 的 `ValueChange<Entity>`。 |
| 连续数值 | `Slider` | 支持轨道点击、拖动和键盘步进；通过 `SliderValue`、`SliderRange`、`SliderStep` 描述状态。 |
| 文字编辑 | `TextInput` | 只对当前 `InputFocus` 实体处理文字、退格、光标、选区和输入法编辑；编辑缓冲区是 `EditableText`。 |
| 列表选择 | `ListBox`、`ListItem` | 支持单选、多选、活动后代和键盘导航，使用 `ValueChange<Entity>` 或列表状态组件。 |
| 菜单 | `MenuButton`、`MenuPopup`、`MenuItem` | 管理菜单打开、关闭、焦点移动和菜单动作，使用 `MenuEvent`。 |
| 对话框 | `Dialog`、`ModalDialog` | 提供对话框拖动、关闭请求、模态遮罩和对话框堆栈行为。 |
| 弹出层 | `Popover` | 根据锚点和放置策略管理弹出内容的位置与显示。 |
| 滚动 | `ScrollArea`、`Scrollbar` | `ScrollArea` 让带 `Overflow::scroll_*` 的节点响应滚轮；`Scrollbar` 提供可拖动的滚动条行为。 |
| 通用辅助 | `ValueChange<T>`、`Activate`、`observe` | Widgets 与应用之间的统一 Entity Event 和 Observer 连接方式。 |

表中的控件都只提供行为，具体的背景、边框、选中标记、滑块和菜单箭头仍由应用自
己创建。当前实验选择前五类作为最常用的最小展示，列表、菜单、对话框、弹出层和独
立滚动条留到需要时再展开。

## 三、用纵向 Flex 组合控件画廊

根实体同时是纵向 Flex 容器和滚动区域：

```rust
(
    ScrollArea,
    Node {
        width: percent(100),
        height: percent(100),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        overflow: Overflow::scroll_y(),
        ..default()
    },
)
```

`flex_direction: FlexDirection::Column` 让标题、说明和各个控件面板从上到下排列。
每个面板也是一个列方向的 Flex 容器，负责把标题、控件和状态文字排列在一起。
`Overflow::scroll_y()` 允许内容超过根节点高度，`ScrollArea` 监听指针滚轮并更新
`ScrollPosition`，所以窗口较小时仍然可以浏览完整画廊。

## 四、画廊中的五种常见控件

| 面板 | 核心组件 | 示例演示 |
| --- | --- | --- |
| Button | `Button` | 点击或按 Enter/Space 产生 `Activate`，Resource 记录激活次数。 |
| Checkbox | `Checkbox`、`Checked` | 点击或按 Enter/Space 切换布尔值，Resource 保存 `checkbox_checked`。 |
| Radio group | `RadioGroup`、`RadioButton` | 鼠标或方向键选择互斥选项，Resource 保存自己的 `RadioChoice`。 |
| Slider | `Slider`、`SliderThumb`、`SliderValue` | 0 到 100 的百分值，Resource 实时保存 `slider_percent`，普通系统同步滑块位置。 |
| Text input | `TextInput`、`EditableText` | 只在获得焦点的输入框中编辑；按 Enter 才把编辑缓冲区提交到 Resource。 |

### 统一的事件和状态模式

控件产生的事件不是最终业务状态。应用可以在 Observer 中把事件转换为自己的资源，
也可以使用 Bevy 提供的 `*_self_update` Observer 先更新控件的状态组件。

例如滑动条的处理同时更新 `SliderValue` 和业务 Resource：

```rust
fn slider_value_changed(
    event: On<ValueChange<f32>>,
    mut state: ResMut<WidgetState>,
    mut commands: Commands,
) {
    state.slider_percent = event.value;
    commands.entity(event.source).insert(SliderValue(event.value));
}
```

`SliderRange::new(0.0, 100.0)` 把值定义为百分比，`SliderStep(1.0)` 定义键盘每次步
进 1。`SliderThumb` 只是标记哪个后代是滑块，应用仍需根据
`range.thumb_position(value.0)` 设置它的 `Node.left`。

复选框可以挂载 `checkbox_self_update`，单选组可以挂载 `radio_self_update`，这样
`Checked` 的互斥关系由 Widgets 维护，而应用 Observer 只负责把选项转换为自己的
枚举或其它业务值。按钮不保存业务值，只发出 `Activate`。

文本输入的编辑状态由 `TextInput` 和 `EditableText` 处理。控件插件通过
`InputFocus` 限定接收输入的 Entity，应用不必自己解析每个字符。示例只在焦点和
Enter 都满足时提交：

```rust
if keyboard.just_pressed(KeyCode::Enter)
    && input_focus.get() == Some(input_entity)
{
    state.committed_text = editable_text.value().to_string();
}
```

因此 `EditableText::value()` 是正在编辑的缓冲区，而 `WidgetState::committed_text`
是确认后的业务值；失去焦点不会自动提交。

## 五、视觉层、控件层和业务层

这个示例把三个层次分开：

| 层次 | 示例内容 | 负责什么 |
| --- | --- | --- |
| 视觉层 | `Node`、颜色、边框、文字和 `SliderThumb`/选中标记子实体 | Flex 布局和用户看到的状态反馈 |
| 控件层 | `Button`、`Checkbox`、`RadioGroup`、`Slider`、`TextInput` | 焦点、鼠标、键盘以及标准 `Activate`/`ValueChange` 行为 |
| 业务层 | `WidgetState` Resource | 激活次数、开关值、选项、百分值和已提交文本，供其它系统读取 |

这种分层让同一种控件可以更换视觉结构，也可以把同一个业务 Resource 同时绑定到多
个控件，而不需要把游戏逻辑写进 UI 的节点树里。
