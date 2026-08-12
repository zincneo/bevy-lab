# 011：样式复用与全局主题

运行示例：

```bash
nix develop
just run ui 011
```

Web 中通常用 CSS class、选择器、继承和 CSS 变量复用样式。Bevy 的核心 UI 没有 CSS 解析器，也没有同等的 class 选择器和级联规则。Bevy 更常见的做法是把样式拆成可复用的场景、组件和资源，再由 ECS 系统负责应用或更新。

## Bevy 中几种样式复用方式

### BSN 场景函数

最简单的复用方式是把固定结构写成返回 `impl Scene` 的函数：

```rust
fn panel() -> impl Scene {
    bsn! {
        Node { padding: UiRect::all(px(18)) }
        BackgroundColor(Color::srgb(0.1, 0.1, 0.15))
    }
}
```

在其它 BSN 场景中调用 `panel()`，就可以复用这组组件。它适合固定的布局和组件组合，但它只是生成场景，不会像 CSS 选择器一样自动匹配已经存在的实体。

### `SceneComponent`：可命名的复合样式/控件

示例中的 `PanelStyle` 和 `PrimaryButtonStyle` 使用 `SceneComponent`：

```rust
#[derive(SceneComponent, Clone, Default)]
struct PanelStyle;

impl PanelStyle {
    fn scene() -> impl Scene {
        bsn! {
            Node { /* 共用布局 */ }
            ThemeBackgroundColor(tokens::PANE_BODY_BG)
        }
    }
}
```

使用时通过 `@PanelStyle` 包含这个场景：

```rust
bsn! {
    @PanelStyle
    Children [ /* 每个面板自己的内容 */ ]
}
```

这相当于一个带语义名称的“样式类”或“可复用组件”。它会在场景生成时展开固定的 `Node`、颜色和其它组件，并且会把 `PanelStyle` 组件本身添加到实体上，方便系统按语义查询。

`SceneComponent` 适合有层级结构的复合 UI。单纯的扁平默认组件可以考虑 Required Components；不需要为了每一个视觉属性都创建一个自定义组件。

## Feathers 的全局主题和设计令牌

当前 Bevy 提供的 Feathers 主题系统是最接近 CSS 全局变量的方案。它包含：

- `UiTheme`：存放当前主题的全局 `Resource`。
- `ThemeToken`：设计令牌，例如窗口背景、主按钮背景、主要文字颜色。
- `ThemeBackgroundColor`、`ThemeBorderColor`、`ThemeTextColor`：把实体的视觉属性绑定到令牌。
- `ThemeContext`：在层级中传递表面层级，让同一个令牌可以根据窗口、面板或浮层上下文得到不同颜色。

示例中的面板并不直接写死 `BackgroundColor`，而是写：

```rust
ThemeBackgroundColor(tokens::PANE_BODY_BG)
ThemeBorderColor(tokens::PANE_HEADER_BORDER)
```

文本和按钮也使用主题令牌。按下 Space 时，示例替换全局 `UiTheme` 资源，所有绑定了令牌的实体会由 Feathers 的主题系统统一更新。这就是全局主题的效果，不需要逐个查询并修改每个面板。

## 它和 CSS class 的区别

Bevy 的组件没有 CSS 那种自动级联：

| Web CSS | Bevy 常用对应方式 |
| --- | --- |
| `.panel { ... }` | `SceneComponent`、场景函数或样式标记组件 |
| CSS 变量 | `UiTheme` 中的 `ThemeToken` 和语义颜色 |
| 选择器匹配已有元素 | `Query<With<StyleMarker>>` 的 ECS 系统 |
| 父元素继承 | `ThemeContext`、Feathers 的文本传播组件或自定义传播系统 |
| 媒体查询/状态选择器 | 根据窗口、状态或交互组件运行系统 |

如果需要根据运行时状态改变样式，可以添加一个语义标记组件并写系统：

```rust
#[derive(Component)]
struct WarningStyle;

fn apply_warning_style(mut query: Query<&mut BackgroundColor, With<WarningStyle>>) {
    for mut color in &mut query {
        color.0 = Color::srgb(0.65, 0.16, 0.12);
    }
}
```

这不是 Bevy 自动提供的 CSS 级联，而是显式的 ECS 数据查询；优点是规则清晰、类型安全，并且可以与游戏状态、资源和系统调度直接组合。

## 选择建议

- 固定结构和固定布局：使用 BSN 场景函数或 `SceneComponent`。
- 多处共用颜色并且希望运行时换主题：使用 Feathers `UiTheme` 和主题令牌。
- 根据状态、交互或业务数据变化：使用语义标记组件和系统查询。
- 大量通用控件的默认外观：可以直接使用 Feathers 提供的主题化控件。

因此，Bevy 目前不是把 CSS 搬进 ECS，而是把“可复用结构”“全局设计令牌”和“运行时样式规则”分别交给场景、资源和系统处理。
