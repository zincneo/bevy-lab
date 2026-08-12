# Lab 003：UI 内容与视觉组件

`Node` 负责 UI 的尺寸、位置和排列，但它本身不负责绘制具体内容。本实验按照一
个 UI 节点从布局到绘制的顺序，认识最常用的内容与视觉组件：

```text
Node 和父子关系
        ↓
节点表面：背景、边框、圆角、阴影
        ↓
节点内容：图片、文字及文字样式
        ↓
布局完成后的变换：平移、缩放、旋转
```

本实验源码中显示到窗口的文本全部使用英文，避免默认字体缺少中文字形。

## 运行示例

```bash
nix develop
just run ui 003
```

示例会显示三个面板：

1. 第一个面板演示背景色、边框、圆角和一个外侧 `BoxShadow`；
2. 第二个面板演示 `ImageNode`；
3. 第三个面板演示文字布局和 `UiTransform`。

图片由启动系统写入 `Assets<Image>`，因此示例不依赖额外的图片文件。

## 一、节点表面

这一组组件决定节点矩形区域本身如何绘制。它们不负责保存文字或图片内容。

### 1. `BackgroundColor`：纯色背景

`BackgroundColor` 是独立的 Component，保存一个 `Color`，用于填充当前 UI 节点的
背景区域：

```rust
BackgroundColor(Color::srgb(0.18, 0.3, 0.48))
```

它只负责颜色，不负责尺寸。节点仍然需要通过 `Node` 得到明确的宽高，或者从父
节点布局中获得尺寸。默认背景色是透明的。

`Color::srgba` 可以设置 alpha。例如，alpha 为 `0.5` 的背景色只会让当前背景半
透明，不会自动让同一 Entity 上的文字或子 Entity 变透明。

### 2. `BorderColor`：边框颜色

`Node::border` 设置四条边的厚度，边框颜色由独立的 `BorderColor` 提供：

```rust
(
    Node {
        border: px(3).all(),
        ..default()
    },
    BorderColor::all(Color::srgb(0.4, 0.7, 1.0)),
)
```

`BorderColor` 可以分别设置 `top`、`right`、`bottom`、`left` 的颜色。只有边框厚
度大于零并且颜色不透明时，边框才会明显显示。

### 3. `Node::border_radius`：圆角

`border_radius` 是 `Node` 的字段，不是独立 Component。它控制四个角的圆角半径：

```rust
Node {
    border_radius: BorderRadius::all(px(12)),
    ..default()
}
```

四个角可以使用相同的半径，也可以分别设置。它通常和 `BackgroundColor`、
`BorderColor`、`BoxShadow` 一起使用。

### 4. `BoxShadow`：节点外侧阴影

`BoxShadow` 是 UI 节点的阴影组件。它内部保存一个按绘制顺序排列的
`Vec<ShadowStyle>`，因此一个节点可以拥有多个外侧阴影。本实验只在一个面板上
展示一个阴影：

```rust
BoxShadow::new(
    Color::srgba(0.0, 0.0, 0.0, 0.35),
    px(8),  // x_offset
    px(8),  // y_offset
    px(0),  // spread_radius
    px(14), // blur_radius
)
```

各参数的作用如下：

| 参数 | 作用 |
| --- | --- |
| `color` | 阴影颜色，alpha 决定阴影透明度。 |
| `x_offset`、`y_offset` | 阴影相对节点的水平和垂直偏移。 |
| `spread_radius` | 阴影向外扩张或向内收缩的范围。 |
| `blur_radius` | 阴影边缘的模糊程度。 |

当前 `BoxShadow` 只提供外侧阴影，没有 CSS `inset` 属性，因此本实验不涉及内侧
阴影或拟态效果。

### 5. 渐变组件（扩展）

当前 Bevy 还提供两个常用的渐变组件：

| 组件 | 用途 |
| --- | --- |
| `BackgroundGradient` | 在节点背景上绘制线性、径向或锥形渐变。 |
| `BorderGradient` | 为节点边框绘制渐变颜色。 |

渐变仍然属于节点表面效果，不能替代 `Node` 的尺寸和布局职责。本实验源码没有
展开渐变配置。

## 二、节点内容

### 1. `ImageNode`：图片内容

`ImageNode` 指定要绘制的图片，`Node` 决定图片占据的区域：

```rust
(
    ImageNode::new(image_handle),
    Node {
        width: px(180),
        height: px(110),
        ..default()
    },
)
```

常用字段如下：

| 字段 | 用途 |
| --- | --- |
| `image` | 要使用的 `Handle<Image>`。 |
| `color` | 对图片进行颜色乘法，也可以通过 alpha 调整图片透明度。 |
| `flip_x`、`flip_y` | 水平或垂直翻转图片。 |
| `rect` | 只显示纹理中的某个矩形区域。 |
| `texture_atlas` | 配合图集选择其中的一个区域。 |
| `image_mode` | 控制图片适应节点尺寸的方式，例如拉伸、保持比例或切片。 |

本实验在启动时创建了一个内存 `Image` 并加入 `Assets<Image>`，所以不需要外部
图片文件。实际项目中通常使用 `AssetServer` 加载图片：

```rust
let image = asset_server.load("images/panel.png");
commands.spawn((
    ImageNode::new(image),
    Node {
        width: px(180),
        height: px(110),
        ..default()
    },
));
```

### 2. 文字组件组合

UI 文字不是由一个组件独立完成的，而是由内容、字体、颜色和文字布局等组件共同
描述。最常用的组合如下：

| 组件 | 职责 |
| --- | --- |
| `Text` | 保存文字内容。 |
| `TextFont` | 指定字体资源和字号。 |
| `TextColor` | 指定文字颜色和透明度。 |
| `TextLayout` | 设置文字行的水平对齐和换行方式。 |
| `TextShadow` | 为文字添加偏移阴影。 |
| `TextBackgroundColor` | 设置文字块或文字片段的背景色。 |
| `TextSpan` | 把一段文字拆成可分别设置样式的文字片段。 |

最基本的文字写法是：

```rust
(
    Text::new("Hello UI"),
    TextFont::from_font_size(24.0),
    TextColor(Color::WHITE),
)
```

添加 `Text` 时，Bevy 会自动补齐 UI 文字所需的 `Node`、`TextLayout`、`TextFont`、
`TextColor` 等常用配套组件。需要修改文字时，直接修改 `Text`；需要修改字体或
颜色时，只修改对应组件即可。

#### `TextLayout` 与节点布局的区别

`TextLayout` 只控制文字区域内部的行对齐和换行，不负责把整个文字 Entity 放到
父节点中的位置：

```rust
(
    Node {
        width: percent(100),
        ..default()
    },
    Text::new("Centered text"),
    TextLayout::justify(Justify::Center),
)
```

如果文字节点的宽度正好等于文字本身的测量宽度，内部没有多余空间，水平居中就不
明显。文字 Entity 在父节点中的位置仍然由父节点的 `Node` 属性控制。

#### 文字 Entity 与父子关系

如果 `Text` 和较大的 `Node` 在同一个 Entity 上，Node的 Flex 对齐不会把这段
文字当作一个子项再次居中；文字通常从该 Node 的内容区域起点开始绘制。

如果希望父节点控制文字的位置，应把文字放在单独的子 Entity 中：

```text
父 Entity：Node + justify_content + align_items
└── 子 Entity：Node + Text + TextFont + TextColor
```

这一点在 Lab 001 中已经详细说明，本实验只关注文字组件本身。

## 三、布局完成后的变换

`UiTransform` 在 `Node` 完成布局后，对节点的视觉结果进行平移、缩放或旋转：

```rust
UiTransform {
    translation: Val2::px(0, -8),
    scale: Vec2::splat(0.92),
    rotation: Rot2::radians(0.12),
}
```

| 字段 | 作用 |
| --- | --- |
| `translation` | 在布局结果上平移节点；可以使用逻辑像素或百分比。 |
| `scale` | 沿 x、y 轴缩放，负值可以实现翻转。 |
| `rotation` | 绕节点中心旋转。 |

`UiTransform` 不会重新分配父节点的 Flex/Grid 空间。可以把它理解为：先由
`Node` 和父子关系完成布局，再对已经得到的节点结果做视觉变换。因此旋转或缩放
后的节点可能覆盖相邻节点，也可能超出原来的布局区域。

`UiGlobalTransform` 是 Bevy 根据层级和 `UiTransform` 计算出的结果，通常只读取，
不直接修改。

## 四、常用组件速查

下面这些类型在普通 UI 中也很常见，但不是本实验源码的重点。这里先说明用途，后
续需要交互、滚动或多相机时再单独展开。

### 布局、层级与显示顺序

| 类型 | 用途 |
| --- | --- |
| `Node` | UI 尺寸、盒模型、Flex/Grid、定位和溢出配置；详见 Lab 002。 |
| `ChildOf`、`Children` | 记录 Entity 父子关系；通常使用 `children![]` 或 `with_children` 建立。 |
| `ScrollPosition` | 保存可滚动节点的水平、垂直滚动偏移，通常和 `Node::overflow` 配合。 |
| `ZIndex` | 调整同一 UI 层级中的前后绘制顺序。 |
| `GlobalZIndex` | 跨 UI 层级设置全局绘制顺序。 |
| `Visibility` | 控制节点是否绘制；父节点隐藏时，子节点也会受到继承可见性影响。 |

### 交互和控件状态

| 类型 | 用途 |
| --- | --- |
| `Button` | 标记一个 Entity 是按钮根节点。当前 `main` 中仍可使用，但属于旧 UI 按钮 API。 |
| `Interaction` | 旧 UI 焦点系统写入的悬停、按下或无交互状态；当前 `main` 已标记弃用。 |
| `RelativeCursorPosition` | 读取光标相对于节点的位置。 |
| `FocusPolicy` | 决定节点是否阻挡后方节点接收指针交互。 |
| `InteractionDisabled` | 将控件标记为不可操作，同时保留布局和绘制。 |
| `Pressed`、`Checked`、`Selected` | 表示控件当前的按住、勾选或选中状态。 |

### UI 运行支持

| 类型 | 用途 |
| --- | --- |
| `UiScale`（Resource） | 统一缩放使用 `Val::Px` 的 UI。 |
| `UiTargetCamera` | 指定 UI 节点渲染到哪一个相机，通常只在多窗口或多 UI 相机时设置。 |
| `ViewportNode` | 把相机视图作为 UI 节点嵌入界面，例如小地图或监控画面。 |

`ComputedNode`、`UiGlobalTransform`、`InheritedVisibility` 等是 Bevy 自动计算或
传播的结果，通常只读取，不作为 UI 样式的配置入口。

## 组件职责总结

一个常见的 UI Entity 可以按下面的职责组合：

```text
Node             尺寸、布局和盒模型
BackgroundColor  节点背景
BorderColor      节点边框颜色
BoxShadow        节点外侧阴影
ImageNode        图片内容
Text             文字内容
TextFont         字体和字号
TextColor        文字颜色
TextLayout       文字内部对齐和换行
UiTransform      布局完成后的平移、缩放和旋转
```

这些组件分别描述 UI 的不同维度。修改背景色不需要重建 Entity，修改文字不需要
改变布局树，修改旋转也不会重新计算兄弟节点的 Flex 空间。
