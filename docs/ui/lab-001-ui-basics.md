# Lab 001：UI 基础元素与实体树

这个实验先完成一件最小但完整的事情：启动 Bevy 窗口，在窗口中显示一个 UI
面板、文本和一个图片节点。随后认识这些元素对应的组件，以及 Bevy 如何通过
Entity 的父子关系组织出树状 UI。

## 运行环境

UI 需要窗口和图形渲染环境。本示例使用 `DefaultPlugins`，它会提供窗口、渲染和
UI 所需的插件；`Camera2d` 是 UI 使用的二维相机。`Startup` 中的 `setup` 系统
只执行一次，用来创建相机和 UI 实体。

```bash
nix develop
just run ui 001
```

运行后会打开一个窗口：深色背景中有一个面板，面板中显示 `Bevy UI` 文本和一块
由 `ImageNode` 绘制的蓝色区域。关闭窗口即可退出程序。

## 最小 UI 元素

Bevy UI 的元素不是一种特殊的对象，而是 World 中的 Entity。Entity 通过添加不同
的组件获得 UI 的布局、外观或内容。

本示例的最外层实体添加了一个 `Node` 和一个 `BackgroundColor`：

- `Node` 是 UI 布局的基础组件，保存尺寸、间距、排列方向、对齐方式等属性。
- `BackgroundColor` 为节点绘制背景色。

`Node` 的宽高设置为 `percent(100)`，表示它占满窗口；`align_items` 和
`justify_content` 让子面板位于根节点的中央。这里使用了最基础的 Flex 布局属性，
只是为了让元素容易观察，Flex 的完整用法会在后续 lab 单独介绍。

## 常用基础组件

| 类型 | 作用 |
| --- | --- |
| `Node` | 描述 UI 节点的布局属性，例如宽高、边距、内边距、排列方向和对齐方式。 |
| `BackgroundColor` | 为节点绘制纯色背景。 |
| `Text` | 保存要显示的文字内容。 |
| `TextFont` | 设置文字使用的字体和字号，也可以指定从资源系统加载的字体。 |
| `TextColor` | 设置文字颜色。 |
| `ImageNode` | 在 UI 节点中显示图片；本示例使用 `ImageNode::solid_color` 生成一个便于观察的纯色图片节点，因此不需要额外的图片资源。 |

这些类型分别负责布局、外观和内容，不要求把所有信息塞进一个组件。后续可以只
修改其中一个组件，例如修改 `Text` 的内容，或修改 `BackgroundColor` 的颜色，
而不用改变实体本身。

`Camera2d` 不是 UI 元素组件，它提供 UI 的显示视角。`DefaultPlugins` 和相机是
让 UI 能够出现在窗口中的运行基础，真正的界面元素仍然由上表中的组件组合而成。

## UI 实体的树状嵌套

大多数 GUI 都会把界面组织成树。Bevy 也使用 Entity 的层级关系表达这种结构：

```text
根 Node（占满窗口）
└── 面板 Node（居中）
    ├── Text（Bevy UI）
    └── ImageNode（蓝色图片区域）
```

在代码中，`children![]` 是创建这种结构的便捷写法。它的实质不是把子实体嵌入
父实体的 `Node`，也不是把一棵 Rust 对象树存进某个 UI 组件，而是让 Bevy 分别
创建独立的 Entity，再用 ECS 的关系组件记录它们之间的 Entity ID：

```text
根实体 E0: Node, BackgroundColor, Children([E1])
面板实体 E1: ChildOf(E0), Node, BackgroundColor, Children([E2, E3])
文本实体 E2: ChildOf(E1), Text, TextFont, TextColor
图片实体 E3: ChildOf(E1), ImageNode, Node
```

实际关系由两个组件配对表达：

- 子实体上的 `ChildOf(parent)` 保存父实体的 Entity ID。它是这组关系的事实来源。
- 父实体上的 `Children` 保存所有子实体的 Entity ID 集合。Bevy 会根据 `ChildOf`
  自动维护它，不应直接修改这个集合，否则可能让父子关系失去同步。

因此，`children![]` 可以理解为“创建一组相关实体并建立父子关系”的语法糖。每个
子节点仍然拥有自己的组件和 Entity ID；父节点只是通过 `Children` 持有指向子节点
的 ID，而不是持有子节点对象本身。根节点、面板和内容节点仍然是独立的 Entity，
只是通过 `ChildOf`/`Children` 连接起来。一个父节点可以拥有多个子节点，这些子
节点互为兄弟节点；任意子节点还可以继续拥有自己的子节点。

建立关系后，Bevy 的层级系统可以从父实体找到子实体，也可以从子实体找到父实体。
UI 布局系统使用这棵树提供布局上下文，实体销毁时层级关系还可以让父实体连同后代
一起处理。这里的父子关系是 Bevy ECS 的通用 Entity 关系，UI 只是利用它来组织
界面树。

父节点的布局会为子节点提供布局上下文，所以本示例的面板可以在根节点中居中，
文本和图片也会按照面板的排列方向放置。父子关系主要描述层级和布局上下文，
并不意味着所有样式组件都会无条件复制给子节点；每个实体仍然可以拥有自己的
`BackgroundColor`、`TextColor` 或其他组件。

## `Text` 作为当前实体组件时的行为

`Text` 本身会自动要求当前 Entity 拥有一个 `Node`，因此下面两种写法表达的是
不同的结构：

```text
写法 A：同一个 Entity
E0: Node + Text("标题") + children

写法 B：Text 是子 Entity
E0: Node + Children([E1])
E1: Node + Text("标题")
```

它们都可以显示文字，但 `Node` 的 Flex 属性作用对象不同：

- `justify_content` 和 `align_items` 负责排列当前节点的直接子 Entity。
- 当 `Text` 和 `Node` 在同一个 Entity 上时，文字是这个 Entity 自身的内容，不是
  `children` 中的一个子项。因此父节点的 Flex 对齐不会把这段文字当作一个子节点
  再居中；文字通常从这个 Node 的内容区域起点开始绘制，看起来就是左上角。
- 当 `Text` 放在单独的子 Entity 上时，它会成为父节点的一个直接子项，父节点的
  `justify_content`/`align_items` 才能控制这个文字节点在父节点中的位置。

你在源码中把 `Text::new("Bevy Content")` 和面板 `Node` 放在同一个 Entity 上，
又把另一个 `Text::new("Bevy UI")` 放在 `children![]` 中，所以前者出现在面板
内容区域的左上角，后者才会参与面板的子节点排列。这是当前输出的预期结果，不是
`Node` 的对齐属性失效。

如果需要让同一个文字节点内部的多行文本水平对齐，可以添加
`TextLayout::justify(Justify::Center)`；它控制文字行在文本区域内部的对齐，不负责
把整个文字 Entity 放到父节点中央，也不负责垂直居中。通常更清晰的做法是使用一个
外层 `Node` 作为容器，再把 `Text` 放进单独的子 Entity，让容器负责位置对齐。

除了 `children![]`，也可以使用 `with_child`、`with_children` 等命令式写法逐步
创建子实体。它们最终表达的都是同一种 Entity 层级关系。

## 示例源码的运行流程

1. `App::new()` 创建应用。
2. `add_plugins(DefaultPlugins)` 注册窗口、渲染和 UI 运行所需的默认插件。
3. `add_systems(Startup, setup)` 注册启动系统。
4. Bevy 运行应用时执行 `setup`，向 World 添加相机、根节点和它的后代实体。
5. UI 插件根据这些实体的 `Node` 和层级关系计算布局，渲染 `BackgroundColor`、
   `Text` 和 `ImageNode`。
6. 窗口保持运行，直到用户关闭窗口，Bevy 收到退出请求后结束应用。

后续实验会在这个最小结构上分别介绍尺寸与间距、Flex/Grid 布局、文本排版、样式
和交互，不会把这些内容全部堆到本实验中。
