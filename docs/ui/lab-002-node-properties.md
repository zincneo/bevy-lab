# Lab 002：Node 组件属性与能力

`Node` 是 Bevy UI 中的核心组件。把 `Node` 添加到 Entity 后，这个 Entity 就具备
了 UI 节点所需的几何、空间、排列、定位、溢出和网格配置能力。`Node` 不负责保存
文字或颜色；这些内容由其它组件提供。

本实验的目标是完整认识 `Node` 的字段：每个字段能解决什么问题、字段使用的类型
如何表示不同的值，以及这些字段之间如何共同决定一个 UI 节点的行为。源码中的
三个面板和一个定位标签只是把一部分字段的效果显示出来，并不把本实验限定为某
一种布局算法。

## 运行示例

```bash
nix develop
just run ui 002
```

窗口中的三个面板分别组合了固定尺寸、百分比尺寸、尺寸约束、盒模型和 Flex 字段；
右下角的紫色标签使用 `PositionType::Absolute`。调整窗口大小可以观察百分比、
尺寸约束和 Flex 分配的变化。

## Node 与 ComputedNode

`Node` 保存的是 UI 节点配置，也就是“这个节点应该具备什么几何和空间行为”的
声明。Bevy 的 UI 系统会结合父节点尺寸、父子关系和子节点内容使用这些字段，
并把最终计算结果写入另一个组件 `ComputedNode`。

```text
Node         -> 你声明的 UI 节点属性
ComputedNode -> Bevy 根据窗口、父子关系和 Node 属性计算出的实际结果
```

`ComputedNode` 中的 `size`、`padding`、`border` 等值已经解析为物理像素，通常只
读取它来观察结果，不直接修改它。`Node` 是输入配置，`ComputedNode` 是计算结果，
两者不要混为一谈。

## 一、显示模型与盒子规则

### `display`：选择节点的显示模型

类型：`Display`。

`display` 决定 Bevy 使用哪一种方式处理节点及其子节点：

- `Display::Flex`：使用 Flex 模型，默认值，也是最常用的选择。
- `Display::Grid`：使用 Grid 模型，行列轨道相关字段才会生效。
- `Display::Block`：使用 Block 模型，按块级内容处理节点。
- `Display::None`：节点和后代不参与布局，也不渲染。

`Display::None` 和 `Visibility` 不同：`Display::None` 会把节点从布局中移除，
而 `Visibility` 可以隐藏绘制但保留布局位置。`Visibility` 是独立组件，不是
`Node` 字段。

### `box_sizing`：决定宽高包含哪些区域

类型：`BoxSizing`。

- `BoxSizing::BorderBox`：`width` 和 `height` 包含 padding 与 border，Bevy 默认使用它。
- `BoxSizing::ContentBox`：`width` 和 `height` 只描述内容区域，padding 与 border 会在外部额外占用空间。

这个字段会影响固定尺寸节点的最终外框大小，尤其需要和 `padding`、`border` 一起
理解。

## 二、尺寸与尺寸约束

### `width`、`height`：期望尺寸

类型：`Val`。

`width` 和 `height` 是节点希望得到的宽度与高度，但不是绝对保证值。父节点可用
空间、Flex/Grid 规则、内容大小以及 min/max 约束都可能影响最终的 `ComputedNode`。

```rust
Node {
    width: px(320),
    height: percent(50),
    ..default()
}
```

### `min_width`、`min_height`：尺寸下限

类型：`Val`。

这两个字段规定节点不能小于的宽度和高度。当 `width` 或布局分配结果更小时，
布局系统会尽量把尺寸保持在最小值以上。适合保证按钮、面板或输入框不会被压缩
到无法使用的大小。

### `max_width`、`max_height`：尺寸上限

类型：`Val`。

这两个字段规定节点不能超过的宽度和高度。常和百分比宽度或 `flex_grow` 一起使用，
让节点可以随父节点变大但不会无限扩张。

### `aspect_ratio`：宽高比

类型：`Option<f32>`。

`Some(value)` 表示 `width / height` 的目标比例，例如 `Some(1.5)` 表示宽度约为
高度的 1.5 倍；`None` 表示不强制宽高比。它适合图片容器、卡片或需要固定比例的
面板。实际尺寸仍会受到显式宽高和 min/max 约束影响。

## 三、节点定位

### `position_type`：是否参与正常排列

类型：`PositionType`。

- `PositionType::Relative`：默认值，节点参与父节点的正常 Flex/Grid/Block 排列。
- `PositionType::Absolute`：节点独立放置在父节点内部，不占用正常流中兄弟节点的空间。

`Absolute` 不是“相对于整个窗口”的意思，而是相对于它的父节点定位；没有合适的
父节点时，位置参照可能不是你期望的结果。

### `left`、`right`、`top`、`bottom`：位置偏移

类型：`Val`。

对 `Absolute` 节点，这四个字段用于指定节点相对于父节点边界的偏移；例如同时
设置 `right: px(24)` 和 `bottom: px(18)` 可以把节点固定在父节点右下角。对
`Relative` 节点，它们是在正常布局位置上进行偏移，不会改变兄弟节点如何计算布局。

参照轴是固定的：`left`/`right` 的百分比相对于父节点宽度，`top`/`bottom` 的
百分比相对于父节点高度。

## 四、盒模型与节点外观几何

> 注意：本项目当前锁定的 Bevy `main` 中，`Node` 明确提供了
> `pub margin: UiRect` 字段。如果在编辑器或其它 Bevy 版本中看不到它，先确认
> 使用的是本项目的 Bevy 版本和正确的 `bevy::ui::Node` 类型。

### 与 Web `div` 的盒模型对照

Bevy 的 `Node` 和 Web 的 `div` 都可以用“盒模型”理解，但它们不是同一个 API。
Web 把样式写在 CSS 中，Bevy 把对应配置写在 `Node` 字段或其它 ECS Component
中。按照从外到内的顺序，可以这样对照：

```text
外部空间       margin       Node::margin: UiRect
边框           border       Node::border: UiRect + BorderColor
边框圆角       border-radius Node::border_radius: BorderRadius
内部空间       padding      Node::padding: UiRect
内容区域       content      子 Entity、Text、ImageNode 等
```

| 从外到内 | Web `div` | Bevy UI | 作用 |
| --- | --- | --- | --- |
| 1 | `margin` | `Node::margin: UiRect` | 元素外部与父布局或兄弟元素之间的空间。背景和边框不会绘制到这块空间。 |
| 2 | `border` | `Node::border: UiRect` + `BorderColor` | 元素边缘的厚度和颜色；Bevy 的厚度与颜色分开存储。 |
| 3 | `border-radius` | `Node::border_radius: BorderRadius` | 对四个角进行圆角处理。 |
| 4 | `padding` | `Node::padding: UiRect` | 边框与内容之间的内部空间。 |
| 5 | content box | 子 Entity、`Text`、`ImageNode` 等 | 实际放置内容的区域。 |

例如，Web 中可以写：

```html
<div class="panel">内容</div>
```

```css
.panel {
    width: 320px;
    margin: 12px;
    border: 3px solid #579;
    border-radius: 10px;
    padding: 16px;
}
```

Bevy 中对应的是同一个 Entity 上的组件组合：

```rust
(
    Node {
        width: px(320),
        margin: px(12).all(),
        border: px(3).all(),
        border_radius: BorderRadius::all(px(10)),
        padding: px(16).all(),
        ..default()
    },
    BorderColor::all(Color::srgb(0.3, 0.45, 0.6)),
    children![Text::new("内容")],
)
```

有两个容易混淆的差异：

1. Web CSS 的 `box-sizing` 默认通常是 `content-box`；Bevy `Node` 的默认值是
   `BoxSizing::BorderBox`。因此在 Bevy 中指定的 `width`/`height` 默认已经包含
   padding 和 border。
2. Web 的 `margin` 还受到块格式化上下文中的 margin 折叠等 CSS 规则影响。Bevy
   的 `Node` 使用自己的 UI 布局规则，不能把 Web CSS 的所有边界情况直接套过来；
   这里只对应它们表达的基本空间关系。

### `margin`：边框外的空间

类型：`UiRect`。

`margin` 描述节点边框外部的空间，影响节点与父布局或兄弟节点之间的距离。它
不是节点自身背景的一部分。

### `padding`：边框内的空间

类型：`UiRect`。

`padding` 描述节点边框与自身内容（子节点或文本）之间的空间。增加 padding 会
让内容离边缘更远；在 `BorderBox` 下，它也会占用节点声明的宽高。

### `border`：边框厚度

类型：`UiRect`。

`border` 分别设置左、右、上、下边框的厚度，参与盒模型和布局尺寸。边框颜色不
属于 `Node`，需要使用独立的 `BorderColor` 组件。

### `border_radius`：圆角半径

类型：`BorderRadius`。

它设置四个角的圆角半径，可以统一设置，也可以分别设置：

```rust
border_radius: BorderRadius::all(px(12)),
```

从外到内可以把 Node 的盒模型理解为：

```text
margin（外边距）
└── border（边框）
    └── padding（内边距）
        └── content（子节点或文本内容）
```

## 五、Flex 方向与换行

这些字段主要在 `display: Display::Flex` 时使用。

### `flex_direction`：主轴方向

类型：`FlexDirection`。

- `Row`：沿内联方向排列，通常表现为从左到右。
- `Column`：沿块方向排列，通常表现为从上到下。
- `RowReverse`、`ColumnReverse`：使用相反方向。

主轴方向会影响 `justify_content` 的含义，也会影响 `flex_basis` 和百分比的参照
方向。

### `flex_wrap`：是否换行

类型：`FlexWrap`。

- `NoWrap`：默认值，所有子节点尽量保持在同一行或同一列。
- `Wrap`：空间不足时换到下一行或下一列。
- `WrapReverse`：换行，但使用相反的交叉轴方向。

### `row_gap`、`column_gap`：子节点之间的间隔

类型：`Val`。

`row_gap` 设置行与行之间的间隔，`column_gap` 设置列与列之间的间隔。它们由父
节点控制子节点之间的统一间距，不需要给每个子节点单独设置 margin。

## 六、Flex 与 Grid 的对齐

### `justify_content`：主轴上的整体分布

类型：`JustifyContent`。

它沿 Flex 主轴分配子节点，常用值包括起始、结束、居中、`SpaceBetween`、
`SpaceAround` 和 `SpaceEvenly`。在 Grid 中，它用于分布整个网格的列。

### `align_items`：子节点默认的交叉轴对齐

类型：`AlignItems`。

在 Flex 中，它控制所有子节点沿交叉轴的默认对齐，例如起始、结束、居中、拉伸或
基线对齐。在 Grid 中，它控制子节点在网格区域块轴方向的默认对齐。

### `align_content`：多行或多轨道的整体分布

类型：`AlignContent`。

当 Flex 产生多行，或 Grid 有多条行轨道时，`align_content` 控制这些行作为整体在
交叉轴/块轴上的分布。只有存在多行或多条轨道时，它和 `align_items` 的区别才明显。

### `align_self`：覆盖单个子节点的交叉轴对齐

类型：`AlignSelf`。

它设置在子节点上，用来覆盖父节点的 `align_items` 对该子节点的默认影响。使用
`Auto` 时，子节点继续采用父节点的 `align_items`。

### `justify_items`：Grid 子节点的默认内联轴对齐

类型：`JustifyItems`。

它主要用于 Grid，控制每个子节点在自己网格区域内的默认内联轴对齐。Flex 布局
中通常不使用它来完成主轴排列。

### `justify_self`：覆盖单个 Grid 子节点的内联轴对齐

类型：`JustifySelf`。

它设置在 Grid 子节点上，用来覆盖父节点的 `justify_items`；使用 `Auto` 时采用
父节点的默认值。

## 七、Flex 尺寸分配

### `flex_grow`：分配剩余空间

类型：`f32`。

当父节点主轴存在剩余空间时，`flex_grow` 大于零的子节点会按比例扩张。`0.0`
表示节点不主动领取剩余空间。

### `flex_shrink`：空间不足时收缩

类型：`f32`。

当子节点总尺寸超过父节点可用空间时，`flex_shrink` 决定节点参与收缩的比例。
通常默认值为 `1.0`；设置为 `0.0` 可以表达“不因 Flex 空间不足而收缩”，但仍
可能受到其它约束影响。

### `flex_basis`：Flex 分配前的初始尺寸

类型：`Val`。

它指定节点在主轴上的初始尺寸，然后才进行 grow/shrink 分配。当 `flex_basis` 有
明确值时，它会优先于主轴方向上的 `width` 或 `height`，但仍遵守 min/max 约束。

## 八、溢出、裁剪与滚动预留

### `overflow`：内容超出边界时如何处理

类型：`Overflow`，内部对 x、y 两条轴分别保存一个 `OverflowAxis`。

每条轴可以选择：

- `Visible`：允许内容绘制到节点边界外。
- `Clip`：裁剪超出的内容，但不因为超出内容改变布局尺寸。
- `Hidden`：裁剪超出的内容，并让溢出内容参与尺寸/布局计算后再裁剪。
- `Scroll`：允许滚动超出内容；滚动位置和滚动条仍需相应组件或 Widget。

常用的快捷构造包括 `Overflow::visible()`、`Overflow::clip()`、`Overflow::hidden()`
和 `Overflow::scroll()`，也可以只对 x 或 y 轴设置。

### `scrollbar_width`：为滚动条预留空间

类型：`f32`，单位是逻辑像素。

当某条轴使用滚动相关模式时，这个字段表示布局为滚动条预留的宽度。它不会单独
创建滚动条，也不会替代 `ScrollPosition` 或滚动 Widget。

### `overflow_clip_margin`：调整裁剪边界

类型：`OverflowClipMargin`。

它可以选择以内容盒、内边距盒或边框盒作为裁剪参照，并设置裁剪边界向外扩展的
逻辑像素距离。普通面板通常使用默认值，只有需要精确控制裁剪范围时才修改。

## 九、Grid 行列配置

这些字段只有节点使用 `display: Display::Grid` 时才发挥主要作用。

### `grid_template_rows`、`grid_template_columns`：显式轨道

类型：`Vec<RepeatedGridTrack>`。

它们声明 Grid 中明确存在的行和列轨道，以及每条轨道的尺寸。一个节点可以有多条
行轨道和列轨道，子节点再通过 `grid_row`、`grid_column` 放置其中。

### `grid_auto_rows`、`grid_auto_columns`：隐式轨道尺寸

类型：`Vec<GridTrack>`。

当子节点放置位置超出显式轨道，或自动放置需要创建额外轨道时，这两个字段决定新
生成的行和列使用什么尺寸。

### `grid_auto_flow`：自动放置方向

类型：`GridAutoFlow`。

它决定自动放置的子节点优先填充行还是列，也可以选择稀疏或尝试填补空洞的密集
放置方式。

### `grid_row`、`grid_column`：子节点的网格位置

类型：`GridPlacement`。

这两个字段设置在 Grid 子节点上，表示它从哪一行/列开始，以及跨越多少条轨道。
它们用于把某个子节点放入指定网格区域。

## 十、文本方向

### `direction`：内联轴方向

类型：`InlineDirection`。

- `InlineDirection::Ltr`：从左到右，默认值。
- `InlineDirection::Rtl`：从右到左。

它影响内联轴方向、文本方向以及部分 Flex 的方向解释。普通界面保持默认的 `Ltr`
即可；需要从右到左书写的界面才会主动设置 `Rtl`。

## 十一、Node 字段常用类型

### `Val`：带单位的值

`Val` 用于尺寸、位置、间距、gap 和 `flex_basis` 等字段。它不是裸 `f32`，因为
UI 需要表达不同的参照系：

| 写法 | 含义 |
| --- | --- |
| `Val::Auto` 或 `auto()` | 交给布局系统根据上下文和内容自动计算。 |
| `Val::Px(120.)` 或 `px(120.)` | 120 个逻辑像素，不是物理像素。 |
| `Val::Percent(50.)` 或 `percent(50.)` | 父节点对应轴长度的 50%；没有父节点时参考窗口对应轴。 |
| `Val::Vw(20.)` 或 `vw(20.)` | 视口宽度的 20%。 |
| `Val::Vh(20.)` 或 `vh(20.)` | 视口高度的 20%。 |
| `Val::VMin(20.)` 或 `vmin(20.)` | 视口较小尺寸的 20%。 |
| `Val::VMax(20.)` 或 `vmax(20.)` | 视口较大尺寸的 20%。 |

百分比的参照轴由字段决定：宽高和 min/max 通常分别参考父节点宽高；`left`/`right`
参考父节点宽度；`top`/`bottom` 参考父节点高度；`margin`、`padding` 和 `border`
中的百分比按父节点宽度计算。所有这些值都是逻辑像素/逻辑尺寸，Bevy 会根据窗口
缩放因子换算到物理像素。

### `UiRect`：四条边的值

`margin`、`padding` 和 `border` 都使用 `UiRect`，其中有 `left`、`right`、`top`
和 `bottom` 四个 `Val` 字段：

```rust
UiRect::all(px(12.0))
UiRect::new(px(8.0), px(16.0), px(4.0), px(4.0))
```

`UiRect::all` 四边相同，`UiRect::new` 按左、右、上、下分别设置，也可以使用
`px(12.).all()` 这种简写。四边可以混合不同的 `Val` 单位。

### 枚举、数字、可选值和集合

- `Display`、`BoxSizing`、`PositionType`、Flex/Grid 策略、对齐类型和 `Overflow`
  用枚举或结构体表达有限的节点行为。
- `flex_grow`、`flex_shrink`、`scrollbar_width` 使用 `f32`，分别表示分配比例或
  逻辑像素数量。
- `aspect_ratio` 使用 `Option<f32>`；`None` 表示不强制宽高比。
- Grid 轨道使用 `Vec<RepeatedGridTrack>` 或 `Vec<GridTrack>`，因为行列数量可以
  不固定；单个 Grid 子节点的位置使用 `GridPlacement`。

## Node 字段与其它组件的边界

`Node` 负责节点的几何大小和空间配置，但以下内容不是它的字段：

- `BackgroundColor`、`BorderColor`：负责颜色。
- `Text`、`TextFont`、`TextColor`：负责文字内容和文字样式。
- `Visibility`、`ZIndex`：负责显示状态和层级顺序。
- `ComputedNode`：保存布局系统计算后的结果。
- `ChildOf`、`Children`：保存 Entity 之间的父子关系。

它们可以和 `Node` 组合在同一个 Entity 上，但仍然是独立的 ECS Component。理解
这个边界后，修改节点尺寸、颜色、文本或层级时就能知道应该读写哪个组件。

## 示例与字段对应关系

- 根节点使用 `width`、`height`、`padding`、`flex_direction`、`row_gap` 和 `overflow`。
- 第一个面板使用 `px` 尺寸、`margin`、`padding`、`border`、`border_radius` 和
  `box_sizing`，观察盒模型。
- 第二个面板使用 `percent`、`min_width`、`max_width`、`flex_grow`、`flex_shrink`
  和 `flex_basis`，观察尺寸约束和空间分配。
- 三个面板的父节点使用 `flex_direction: Row`、`align_items` 和 `column_gap`。
- 紫色标签使用 `position_type: Absolute`、`right` 和 `bottom`，因此不挤压正常
  流中的面板。

源码没有为了“展示字段”而强行把 Grid、滚动等能力全部塞入同一个画面；这些字段
已经在本篇按类型和用途完整列出，后续专门的 Grid、滚动和交互实验会使用它们完成
更小、更容易观察的示例。
