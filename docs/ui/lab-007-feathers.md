# Lab 007：使用 Bevy Feathers 主题控件

本实验继续实现 Lab 005/006 的控件画廊，但把手写的按钮、复选框、单选按钮、滑动
条和文本输入框视觉结构替换为 Bevy Feathers 提供的主题控件。控件仍然通过同一套
`Activate`、`ValueChange<T>`、`InputFocus` 和 Resource 机制工作，因此这里只替换
UI 的视觉和静态结构，不改变业务状态逻辑。

## 运行示例

```bash
nix develop
just run ui 007
```

使用 Feathers 需要在 `Cargo.toml` 中启用 Bevy 的可选功能：

```toml
bevy = { git = "https://github.com/bevyengine/bevy.git", branch = "main", features = ["bevy_feathers"] }
```

示例通过 `DefaultPlugins`、`FeathersPlugins` 和默认暗色主题启动：

```rust
App::new()
    .add_plugins((DefaultPlugins, FeathersPlugins))
    .insert_resource(UiTheme(create_dark_theme()))
```

## 一、Feathers 是什么

`bevy_feathers` 是建立在 Bevy UI 和 `bevy_ui_widgets` 之上的**主题化、带样式的控
件层**。它提供：

- 固定的控件视觉结构，例如按钮背景、复选框边框和滑动条填充条；
- 一组设计 Token，例如按钮、滑块、输入框和文本的颜色；
- 默认字体、图标、圆角、焦点指示器和鼠标光标；
- `UiTheme` Resource，用于统一控制颜色和主题上下文；
- 通过 `bsn!` 使用的 `@Feathers...` 场景组件。

它不是另一个 ECS，也不替代底层 Widgets。Feathers 控件内部仍然使用
`Button`、`Checkbox`、`RadioButton`、`Slider` 和 `TextInput`，只是把原本需要应用
手动创建的视觉子实体和样式系统封装起来。

当前 Bevy 源码对 Feathers 的定位是编辑器和 Inspector 风格的功能性 UI。它的默认样
式并不是面向所有游戏的通用主题；如果项目需要完全不同的视觉风格，可以把它作为
参考，或者在同样的 `ui_widgets` 行为层之上创建自己的主题控件。

## 二、Feathers 与 `ui_widgets` 的关系

两层的职责可以这样区分：

| 层 | 主要内容 | 是否提供默认视觉 |
| --- | --- | --- |
| `bevy_ui` | `Node`、Flex、颜色、边框、文字、滚动和布局 | 提供渲染能力，不提供完整控件主题 |
| `bevy_ui_widgets` | `Button`、`Checkbox`、`Slider`、`TextInput` 等行为组件 | 不提供固定样式，headless |
| `bevy_feathers` | `FeathersButton`、`FeathersCheckbox`、`FeathersSlider` 等主题场景组件 | 提供默认视觉结构、字体和主题系统 |

因此 007 中的事件处理仍然和 005/006 一样：

```rust
fn slider_value_changed(
    event: On<ValueChange<f32>>,
    mut state: ResMut<WidgetState>,
) {
    state.slider_percent = event.value;
}
```

换成 Feathers 不会改变 `ValueChange<f32>` 的含义，也不会改变滑动条的业务状态管理。

## 三、当前 Feathers 提供的内容

以下列表以本项目当前使用的 Bevy `main` 源码为准。Feathers 目前不只是 005/006
中使用的五种控件，而是包含交互控件、被动容器、显示辅助函数以及主题基础设施。

### 1. 交互控件

这些内容位于 `bevy::feathers::controls`，通常通过 `bsn!` 中的
`@Feathers...` 场景组件使用。普通控件内部使用 `bevy_ui_widgets` 的行为组件，
而颜色选择器等控件还会结合 Picking 和 UI Material；Feathers 负责提供默认的结构
和视觉样式。

| 控件 | 用途 |
| --- | --- |
| `FeathersButton`、`FeathersToolButton` | 普通按钮和适合工具栏、面板标题栏的紧凑按钮；按钮支持 `Normal`、`Primary`、`Plain` 三种 `ButtonVariant`。 |
| `FeathersCheckbox` | 带标签的复选框，表示独立的开/关选项。 |
| `FeathersRadio` | 单选项；通常与 `RadioGroup` 一起使用实现互斥选择。 |
| `FeathersToggleSwitch` | 开关样式的布尔控件；底层行为仍是 `Checkbox`。 |
| `FeathersSlider` | 普通数值滑动条，支持最小值、最大值、方向和步进等底层滑块配置。 |
| `FeathersColorSlider` | 用于编辑颜色某个通道的滑动条，支持 RGB、HSL 和 Alpha 通道。 |
| `FeathersColorPlane` | 二维颜色选择平面，可选择 RGB 平面或 HSL 的色相/饱和度、色相/亮度平面。 |
| `FeathersColorSwatch` | 显示颜色的小色块，可表现透明度和棋盘格背景。 |
| `FeathersTextInputContainer`、`FeathersTextInput` | 输入框外部装饰容器和实际文本输入实体；支持可见宽度和最大字符数配置。 |
| `FeathersNumberInput` | 数值输入控件，支持 `f32`、`f64`、`i32`、`i64`，以及步进、精度、软限制、硬限制和范围回绕。 |
| `FeathersScrollbar` | 与某个可滚动实体绑定的水平或垂直滚动条，可配置内容不足时是否自动隐藏。 |
| `FeathersListView`、`FeathersListRow` | 列表及其列表行，支持选择、悬浮和滚动相关的列表行为。 |
| `FeathersSelect` | 下拉选择控件，内部组合菜单、弹出层和列表；也提供 `list_rows_from_strings` 辅助函数。 |
| `FeathersMenu`、`FeathersLazyMenu` | 菜单及延迟创建弹出内容的菜单。 |
| `FeathersMenuButton`、`FeathersMenuToolButton` | 打开菜单的普通按钮和工具按钮。 |
| `FeathersMenuPopup`、`FeathersMenuItem`、`FeathersMenuDivider` | 菜单弹出层、菜单项和菜单分隔线。 |
| `FeathersDialog`、`FeathersFloatingDialog` | 模态对话框和可浮动窗口式对话框。 |
| `FeathersDialogHeader`、`FeathersDialogClose`、`FeathersDialogBody`、`FeathersDialogFooter` | 对话框的标题栏、关闭按钮、正文和底部区域。 |
| `FeathersDisclosureToggle` | 展开/折叠内容时使用的箭头指示控件。 |
| `VirtualKeyboard<T>` | 根据键盘布局动态生成的虚拟键盘，并通过 `VirtualKeyPressed<T>` 报告按键。 |

控件还提供了配套的配置和状态类型，例如：

- 按钮：`ButtonVariant`、`FeathersButtonProps`。
- 颜色控件：`ColorChannel`、`ColorPlaneValue`、`SliderBaseColor`、
  `ColorSlider`、`FeathersColorSliderProps`、`ColorSwatchValue`、
  `ColorSwatchFg`、`FeathersColorSwatchProps`。
- 数值输入：`NumberFormat`、`NumberInputValue`、`NumberInputWrap`、
  `NumberInputRange`、`SoftLimit`、`HardLimit`、`NumberInputPrecision`、
  `NumberInputStep`、`FeathersNumberInputProps`。
- 列表和选择：`FeathersListViewProps`、`FeathersSelectProps`、`OptionIndex`、
  `ScrollbarGutter`。
- 对话框：`FeathersDialogProps`、`FeathersFloatingDialogProps`。
- 菜单：`FeathersMenuButtonProps`、`FeathersMenuItemProps`。
- 滑块、滚动条和文本输入：`FeathersSliderProps`、`FeathersScrollbarProps`、
  `FeathersTextInputProps`。
- 虚拟键盘：`VirtualKeyboardProps<T>` 和 `VirtualKeyPressed<T>`。

源码还保留了 `button_bundle`、`checkbox_bundle`、`radio_bundle`、`slider_bundle`、
`toggle_switch_bundle`、`color_plane_bundle`、`color_slider_bundle`、
`color_swatch_bundle` 和 `virtual_keyboard_bundle` 等旧版 Bundle 构造函数，但这些
接口已经标记为 deprecated，新代码应使用对应的 BSN 场景组件。

因此，007 示例只选取了 Button、Checkbox、Radio、Slider 和 TextInput 来对比
005/006 的代码量，并不代表 Feathers 只有这五种控件。

### 2. 被动容器

这些内容位于 `bevy::feathers::containers`，本身主要负责组织布局和主题层级，
不负责输入行为：

| 内容 | 用途 |
| --- | --- |
| `flex_spacer()` | 在 Flex 布局中占据剩余空间。 |
| `group()`、`group_header()`、`group_body()` | 创建带有标题和正文区域的控件分组。 |
| `pane()`、`pane_header()`、`pane_header_divider()`、`pane_body()` | 创建面板及其标题、分隔线和正文区域。 |
| `subpane()`、`subpane_header()`、`subpane_body()` | 创建嵌套在面板中的次级面板。 |

### 3. 显示辅助内容

这些内容位于 `bevy::feathers::display`，用于显示文本和图标，不属于交互控件：

- `caption()`：控件内部的普通说明文字。
- `label()`：主要标签文字。
- `label_dim()`：较弱的辅助说明文字。
- `label_small()`：较小字号的标签文字。
- `icon()`：使用主题颜色显示嵌入式图标。
- `icon_untinted()`：显示不使用主题着色的嵌入式图标。
- `ThemedIcon`：标记需要由主题系统更新的图标实体。

当前源码内置了 Fira Sans、Fira Mono 字体，以及 `chevron-down`、`chevron-right`、
`x` 三个常用图标。

### 4. 主题和通用样式基础设施

Feathers 还提供了用于统一控件风格的基础内容：

- `FeathersPlugins`：一次性注册焦点、光标、主题、所有控件及其样式系统。
- `FeathersCorePlugin`：注册主题、字体、光标、焦点框和控件核心系统；通常由
  `FeathersPlugins` 间接添加。
- `ControlsPlugin`：注册所有 Feathers 控件插件；通常由 `FeathersCorePlugin` 间接添加。
- `UiTheme`、`ThemeProps`：保存当前主题和颜色映射。
- `create_dark_theme()`：创建官方默认暗色主题。
- `ThemeToken`、`SemanticToken`、`SurfaceLevel`：分别表示控件设计 Token、语义颜色
  Token 和窗口、面板、分组、浮层等主题层级。
- `ThemeBackgroundColor`、`ThemeBorderColor`、`ThemeTextColor`、
  `InheritableThemeTextColor`、`ThemeContext`：将主题颜色应用到背景、边框和文本，
  并支持沿 UI 父子树继承。
- `InheritableFont`：向子文本实体继承字体、字号和字重。
- `FocusIndicator`、`FocusWithinIndicator`：显示实体自身或其子树获得焦点时的焦点框。
- `DefaultCursor`、`EntityCursor`、`OverrideCursor`：设置默认、按实体变化以及临时覆盖的鼠标光标。
- `RoundedCorners`：为按钮、菜单和分段控件选择哪些角需要圆角。
- `palette`、`constants`、`tokens`：提供默认颜色、字体路径、尺寸常量和控件设计 Token。

这些基础设施不会替代 `Node`、`Text`、`Children` 等 Bevy UI 基础组件，而是把它们
组合成一套可以复用的主题化 UI。

## 四、使用 Feathers 场景组件

### 1. Button

手写版本需要自己创建按钮节点、文本子实体、边框、背景、焦点和 hover 状态；Feathers
版本只需要：

```rust
(
    @FeathersButton {
        @caption: bsn! { caption("Activate button") }
    }
    on(button_activated)
)
```

`@FeathersButton` 会生成底层 `Button`、必要的 `Node`、焦点指示器、主题背景和文字
继承组件。`@caption` 是按钮内部显示的场景列表。

### 2. Checkbox

```rust
(
    @FeathersCheckbox {
        @caption: bsn! { caption("Enable feature") }
    }
    on(checkbox_self_update)
    on(checkbox_changed)
)
```

Feathers 已经创建了复选框边框、选中标记和主题文本。应用仍可以通过
`checkbox_self_update` 更新 `Checked`，再通过自己的 Observer 更新业务 Resource。

### 3. RadioGroup 和 Radio

Radio 组的分组关系仍然由底层 `RadioGroup` 负责，单个选项使用 Feathers 的
`@FeathersRadio`：

```rust
(
    RadioGroup
    on(radio_self_update)
    Children [
        (
            @FeathersRadio {
                @caption: bsn! { caption("First option") }
            }
            Checked
        ),
        @FeathersRadio {
            @caption: bsn! { caption("Second option") }
        },
    ]
)
```

Feathers 负责圆形边框、内部标记、焦点和主题文字，`RadioGroup` 仍然负责互斥选择
和 `ValueChange<Entity>`。

### 4. Slider

```rust
(
    @FeathersSlider {
        @min: 0.0,
        @max: 100.0,
    }
    SliderValue(50.0)
    SliderStep(1.0)
    on(slider_self_update)
    on(slider_value_changed)
)
```

FeathersSlider 已经包含滑轨、动态填充条和当前值文本，不再需要手动添加
`SliderThumb`，也不需要普通系统根据 `SliderRange::thumb_position` 修改 thumb 位置。

### 5. TextInput

Feathers 的文本输入通常由一个容器和一个输入实体组成：

```rust
(
    @FeathersTextInputContainer
    Children [(
        @FeathersTextInput
        DemoTextInput
    )]
)
```

容器提供背景、边框和焦点范围，输入实体提供 `TextInput`、`EditableText`、字体、光
标和主题颜色。应用仍可以使用 `InputFocus` 判断当前输入框是否聚焦，并在 Enter 时
把 `EditableText::value()` 写入自己的缓存 Resource。

## 五、这次减少了哪些代码

当前项目中，两个版本的 Rust 行数约为：

| 示例 | 行数 | 主要内容 |
| --- | ---: | --- |
| 005 | 657 | 手动创建所有控件视觉子实体和状态样式系统 |
| 006 | 627 | 使用 `bsn!` 声明静态实体树，但仍手动声明控件视觉结构 |
| 007 | 385 | 使用 Feathers 主题控件，只保留布局、业务状态和提交逻辑 |

007 相比 006 少了约 38% 的代码，主要减少的是：

- `CheckboxMark` 和 `RadioMark` 等自定义视觉实体；
- 按钮、复选框、单选框的 hover、pressed、checked 样式系统；
- 滑动条轨道、填充条、thumb 和 thumb 位置计算；
- 文本输入框的边框、字体、光标主题和焦点视觉；
- 大量颜色、边框和字体配置。

没有减少的部分包括：

- `WidgetState` 业务 Resource；
- Button、Checkbox、Radio、Slider 的事件 Observer；
- 文本输入提交规则；
- 状态标签和业务数据显示；
- 页面本身的 Flex 布局和面板结构。

这说明 Feathers 解决的是“控件主题和视觉实现过于繁琐”，并不会替应用决定业务状
态如何流动。

## 六、Feathers 的适用范围

适合直接使用 Feathers 的场景：

- 编辑器、Inspector、工具面板和调试界面；
- 希望快速拥有一致的基础控件样式；
- 不想重复实现按钮、复选框、滑动条和输入框的视觉状态。

如果需要完全自定义的游戏 HUD、卡通界面或特殊交互，通常有两种选择：

1. 继续使用 `ui_widgets`，自己编写主题和视觉结构；
2. 参考 Feathers 的场景组件和主题 Token，复制出适合项目风格的控件层。

因此，`ui_widgets` 更接近控件行为基础设施，Feathers 更接近一个可直接使用的默认
主题层，而 `bsn!` 负责把这些静态 UI 结构组织成可读的 Entity 场景。
