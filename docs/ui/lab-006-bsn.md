# 006：BSN（Bevy Scene Notation）完整语法概览

运行示例：

```bash
nix develop
just run ui 006
```

本实验把 Lab 005 的控件画廊改写为 BSN 场景。BSN 用一段接近实体树的声明描述：

- 要创建哪些实体；
- 每个实体有哪些组件以及组件的初始值；
- 实体之间有什么关系；
- 哪些场景需要组合、复用或附加 Observer。

BSN 不是 HTML，也不是另一套 UI 框架。它最终仍然创建 Bevy 的 Entity、Component、
Children 和 Observer；它只是把静态实体结构从一连串 `Commands` 调用改成了可组合的场景声明。

## 1. 从场景到 World

`bsn!` 产生一个实现 `Scene` 的值。一个 `Scene` 描述一个根实体及其组件和相关实体，
必须通过场景 API 才会真正进入 `World`：

```rust
fn panel() -> impl Scene {
    bsn! {
        Node { width: px(320), height: px(80) }
        BackgroundColor(Color::srgb(0.1, 0.1, 0.1))
        Children [Text::new("A static child")]
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn_scene(panel());
}
```

常用生成入口有：

| API | 用途 |
| --- | --- |
| `world.spawn_scene(scene)` | 在独占访问 `World` 时立即解析并生成场景 |
| `commands.spawn_scene(scene)` | 在普通系统中提交生成命令；命令应用时生成场景 |
| `world.queue_spawn_scene(scene)` | 等待场景依赖的资源加载完成后再生成 |
| `commands.queue_spawn_scene(scene)` | 普通系统中的异步依赖版本 |

没有资源依赖时，立即版本最简单；场景引用了还没加载的图片、字体等资源时，使用 queued
版本。无论使用哪个入口，`bsn!` 本身都只是场景描述，不会自动运行 App 或系统。

## 2. 一个实体中的场景条目

在同一个场景层级中，空白分隔的条目作用于同一个实体：

```rust
bsn! {
    Node { width: px(200) }
    BackgroundColor(Color::BLACK)
    Button
}
```

这相当于把 `Node`、`BackgroundColor` 和 `Button` 一起放入一个 `spawn` 元组。

### 组件的三种常见写法

```rust
// 单元组件或无参数组件：使用 Default
Button

// 元组组件：给出部分或全部字段
SliderValue(50.0)
TabIndex(0)

// 结构体组件：可以只覆盖需要修改的字段
Node {
    width: px(320),
    flex_direction: FlexDirection::Column,
}
```

也可以使用模块路径、字段简写和枚举变体：

```rust
ui_widgets::Button
Node { width }
Visibility::Hidden
```

字段没有写出的部分不会被随意清空，而是保留之前的补丁值；如果之前没有值，则使用
类型默认值。这是 BSN 的“补丁”语义，详见第 7 节。

要让自定义组件可以直接出现在 BSN 中，通常派生：

```rust
#[derive(Component, Default, Clone)]
struct DemoMarker;
```

如果组件字段需要场景生成时的上下文，例如根据路径由 `AssetServer` 生成句柄，使用
`FromTemplate`：

```rust
#[derive(Component, FromTemplate)]
struct Icon {
    image: Handle<Image>,
}

bsn! {
    Icon { image: "icons/start.png" }
}
```

枚举组件需要每个变体都有默认构造方式。可以使用 `VariantDefaults` 生成变体默认函数，
或者直接使用 `FromTemplate`。普通 UI 实验一般只使用 `Default + Clone` 即可。

### BSN 前缀速查

| 写法 | 含义 |
| --- | --- |
| `Component`、`Component(...)`、`Component { ... }` | 添加或补丁一个组件 |
| `scene()`、`{expr}` | 包含一个 `Scene` |
| `@Widget`、`@Widget { @prop: value }` | 包含一个 `SceneComponent`，并设置 scene props |
| `~Template`、`~Template { field: value }` | 把类型当作 `Template` 使用 |
| `#Name`、`#{expr}` | 给实体命名并建立场景内引用 |
| `on(handler)` | 给当前实体附加 Entity Observer |
| `Children [...]`、`MyRelation [...]` | 用关系生成多个相关实体 |
| `:scene`、`:"file.bsn"` | 请求缓存的场景；函数缓存和官方 `.bsn` 资源目前有限制 |
| `{ expr }` | 在值位置插入 Rust 表达式 |

## 3. `Children` 和关系

`Children [...]` 会为当前实体生成相关的子实体：

```rust
bsn! {
    Node { flex_direction: FlexDirection::Column }
    Children [
        Text::new("Title"),
        (
            Node { width: px(200) }
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2))
        ),
    ]
}
```

方括号中的逗号分隔实体Entity；同一项中用空白分隔的内容仍属于同一个子实体。括号通常可以
省略，但在内容复杂或需要明确“一个实体”时建议保留。子实体还可以继续拥有自己的
`Children [...]`，从而构建完整的 UI 树。

除了内置的 `Children`，BSN 也支持其它实现 `RelationshipTarget` 的关系：

```rust
Followers [
    #GruntA Grunt,
    #GruntB Grunt,
]
```

如果要直接把当前实体挂到已有实体上，可以使用：

```rust
ChildOf(parent_entity)
```

`ChildOf` 的参数可以是 `Entity`，也可以是同一个场景作用域中用 `#Name` 声明的实体引用。

## 4. 场景组合

任何返回 `impl Scene` 的函数都可以嵌入其它场景：

```rust
fn panel() -> impl Scene {
    bsn! {
        Node { padding: UiRect::all(px(18)) }
    }
}

fn root() -> impl Scene {
    bsn! {
        Node
        Children [panel(), panel()]
    }
}
```

也可以使用 `{ expression }` 显式插入一个 `Scene`：

```rust
let content = panel();
bsn! {
    Node
    Children [{content}]
}
```

`scene()`、`scene(value)` 是用于包含场景函数的简写形式；普通函数调用和大括号表达式
都可以完成同样的组合工作。场景组合不是简单的文本拼接，而是把各个场景产生的补丁按
出现位置依次合并。

## 5. `SceneComponent`：可命名的复合场景

当一个场景片段需要一个稳定的语义名称，并且希望它自己就是一个可复用的层级组件时，
可以派生 `SceneComponent`：

```rust
#[derive(SceneComponent, Clone, Default)]
struct PanelStyle;

impl PanelStyle {
    fn scene() -> impl Scene {
        bsn! {
            Node { padding: UiRect::all(px(18)) }
            BackgroundColor(Color::srgb(0.1, 0.1, 0.15))
        }
    }
}

bsn! {
    @PanelStyle
    Children [Text::new("Panel content")]
}
```

`@PanelStyle` 会包含 `PanelStyle::scene()` 的内容，并把 `PanelStyle` 组件本身添加到
实体上。这样既能复用结构，也能让普通 `Query<With<PanelStyle>>` 找到这些实体。

默认情况下，派生宏会调用同名类型的 `scene()`；也可以指定场景函数：

```rust
#[derive(SceneComponent, Default, Clone)]
#[scene(panel_scene)]
struct PanelStyle;

fn panel_scene() -> impl Scene {
    bsn! { Node }
}
```

### Scene props

需要参数化场景时，使用 `#[scene(PropsType)]` 和 `@` 前缀的 prop：

```rust
#[derive(SceneComponent, Default, Clone)]
#[scene(ListProps)]
struct List;

#[derive(Default)]
struct ListProps {
    count: usize,
}

impl List {
    fn scene(props: ListProps) -> impl Scene {
        let items = (0..props.count).map(|i| bsn! {
            Text::new(format!("Item {i}"))
        }).collect::<Vec<_>>();
        bsn! {
            Node
            Children [{items}]
        }
    }
}

bsn! {
    @List { @count: 3 }
}
```

`@count` 是场景参数；普通的 `count: value` 则是给 `List` 组件自身字段打补丁。两者
可以同时存在，但 prop 会在场景包含时立即用于生成内部场景。

## 6. 命名实体和实体引用

`#Name` 会给当前实体添加 `Name("Name")`，并在当前场景作用域中建立引用：

```rust
bsn! {
    #Panel
    Node
    Children [
        #Title Text::new("Title"),
        Link(#Panel),
    ]
}
```

也可以手动写 `Name("Custom name")`，或者让名称来自表达式：

```rust
bsn! {
    #{name}
    Name("Visible name")
}
```

`#Title`、`#Panel` 是场景构建时的引用，不是运行时 `Entity` 的永久别名。引用要传给
组件字段时，该字段需要支持 `FromTemplate`，BSN 才能把 `EntityTemplate` 解析为实体。

## 7. 补丁和组合顺序

BSN 不要求每次都把组件的所有字段重新写一遍。后出现的条目只修改它明确写出的字段：

```rust
bsn! {
    Node { width: px(100), height: px(300) }
    Node { width: px(200) }
}
```

最终结果是 `width = 200`、`height = 300`。场景函数也遵循同样的顺序：

```rust
bsn! {
    base_panel()
    Node { width: px(640) }
}
```

这会复用 `base_panel()` 的其它字段，只覆盖宽度。补丁是按场景条目顺序合并的，因此
可以把基础样式放在前面，把局部覆盖放在后面。需要注意：同一实体中的条目是补丁，
`Children [...]` 中的逗号则表示创建不同实体，不是覆盖同一个组件。

## 8. Observer

`on(handler)` 会把一个 Entity Observer 附加到当前实体；Observer 的第一个参数类型决定
它监听哪一种 `EntityEvent`：

```rust
(
    Button
    on(button_activated)
)

fn button_activated(event: On<Activate>, mut state: ResMut<AppState>) {
    info!("activated {:?}", event.entity);
    state.count += 1;
}
```

也可以直接写闭包：

```rust
bsn! {
    Button
    on(|event: On<Activate>, mut query: Query<&mut Pressed>| {
        if let Ok(mut pressed) = query.get_mut(event.entity) {
            pressed.0 = true;
        }
    })
}
```

每个 `on(...)` 都会添加一个独立 Observer；它可以使用普通系统参数访问 `Query`、
`Resource`、`Commands` 等 ECS 数据。BSN 只负责注册 Observer，Observer 的执行仍由事件
触发时的 ECS 流程负责。

## 9. 场景列表：`bsn_list!`

`bsn!` 始终描述一个根实体；需要同时生成多个没有共同父实体的根实体时使用 `bsn_list!`：

```rust
let scenes = bsn_list![
    #First Text::new("First"),
    #Second Text::new("Second"),
];

commands.spawn_scene_list(scenes);
```

列表中的实体使用逗号分隔；同一个实体的多个组件仍然使用空白分隔。列表中的所有根
实体共享一个命名作用域，因此可以相互引用。

`SceneList` 也可以嵌入关系列表中：

```rust
fn container(items: impl SceneList) -> impl Scene {
    bsn! {
        Node
        Children [
            #Header Text::new("Header"),
            {items},
            #Footer Text::new("Footer"),
        ]
    }
}
```

这里的 `{items}` 会展开列表中的多个实体。如果想把一个 `Scene` 当成一个实体而不是
展开列表，需要用括号包住表达式：`({one_scene})`。

## 10. 动态值和 Rust 表达式

BSN 不是只能写常量。场景函数是普通 Rust 函数，可以接收参数、捕获变量，并在值位置
使用 `{ ... }` 插入任意 Rust 表达式：

```rust
fn health_bar(current: f32, max: f32) -> impl Scene {
    bsn! {
        Node { width: {px(300.0 * current / max)} }
        Health { current, max }
    }
}
```

简单的局部变量通常可以直接写；复杂表达式、运算、字符串拼接和需要避免宏歧义的内容
使用大括号：

```rust
let label = format!("Score: {score}");
bsn! {
    Text::new({label})
    Score({score})
}
```

BSN 语法本身还没有独立的 `if`/`match` 条目。条件内容应在宏外计算，或放入表达式块：

```rust
let color = if selected { Color::srgb(0.2, 0.6, 1.0) } else { Color::GRAY };
bsn! { BackgroundColor({color}) }
```

完全不同的场景可以先放入 `Box<dyn Scene>`，再通过 `{scene}` 插入。数据在场景生成后
才变化时，BSN 不会自动响应；应由系统使用 `Commands` 创建、删除或修改实体，详见 UI 010。

## 11. `Template`、`template_value` 和上下文

BSN 中的组件值通常是一个可复制的模板。需要把已经存在的组件实例传入场景时，使用
`template_value`：

```rust
let transform = Transform::from_xyz(10.0, 20.0, 0.0);
bsn! {
    template_value(transform)
}
```

需要自定义生成逻辑或访问 `World` 时，可以使用 `template` 闭包：

```rust
bsn! {
    template(|context| {
        let config = context.resource::<UiConfig>();
        TooltipText(config.help.clone())
    })
}
```

`TemplateContext` 提供当前实体、`World` 资源和命名实体引用等场景生成上下文。更复杂
的可复用模板可以实现 `Template`，或者为组件派生 `FromTemplate`。`~Type` 前缀用于明确
把一个类型当作 `Template` 使用，而不是普通 Component：

```rust
bsn! {
    ~MyTemplate
    ~MyTemplate { value: 3 }
}
```

这是 BSN 最灵活的扩展点，但日常 UI 通常优先使用 `Default + Clone`、场景函数或
`SceneComponent`。

## 12. 资源路径和依赖

当某个字段是 `Handle<T>`，并且拥有 `FromTemplate` 能力时，可以直接写资源路径：

```rust
bsn! {
    ImageNode { image: "ui/icons/start.png" }
}
```

BSN 会把路径转换成场景依赖，在依赖加载前不能安全解析场景。因此有资源依赖的场景通常
使用 `queue_spawn_scene`。资源路径只解决句柄加载，不会让 BSN 变成完整的外部 UI 文件格式。

## 13. 缓存和 `.bsn` 文件现状

前缀 `:` 表示希望缓存一个场景：

```rust
bsn! {
    :base_panel()
    Node { width: px(640) }
}
```

当前 Bevy 主分支的缓存仍有明确限制：场景资源缓存可用，但函数场景和
`SceneComponent` 的缓存尚未完整接通，不能把所有 `:scene()` 写法都当成可用功能。

官方 `.bsn` 资产格式也尚未发布。虽然语法中预留了 `:"ui/panel.bsn"`，但当前不能直接
把 `.bsn` 文件放入 `assets/` 后加载。现在需要把 BSN 写在 Rust 中，或者自己实现资源格式
和 `AssetLoader`。

## 14. 006 示例中的职责划分

006 使用的结构可以概括为：

```text
setup
└── commands.spawn_scene(demo_root())
    └── demo_root / panel / button_panel / ...   BSN 静态实体树

Update 和 Observer
└── 读取输入、Resource、Query
    └── 修改文字、颜色、滑块值和控件状态
```

适合写进 BSN 的内容：

- 固定的实体层级和父子关系；
- 初始布局、颜色、字体和控件组件；
- 可复用的场景函数和 `SceneComponent`；
- 固定的 Observer 注册；
- 已知的资源路径和场景依赖。

仍然应该交给普通 System 或 Commands 的内容：

- 每帧或按状态变化修改组件；
- 运行时决定的子实体数量；
- 动态创建、删除和重排 UI；
- 读取输入、Resource、Message 或其它 ECS 数据；
- 需要在场景生成后才知道的内容。

BSN 的完整使用方式可以归纳为：用场景条目描述实体，用 `Children` 或其它关系组织
实体，用函数和 `SceneComponent` 组合场景，用补丁覆盖局部字段，用模板处理生成上下文，
再由普通 ECS 系统负责运行时行为。
