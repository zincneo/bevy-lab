# 010：混合 BSN 静态 UI 与动态内容

运行示例：

```bash
nix develop
just run ui 010
```

本实验解决一个常见场景：面板的标题、边框、间距和布局是固定的，适合用 `bsn!` 写成静态场景；面板内部的条目数量和文字来自运行时资源，无法在编译时写死。这时不要放弃 BSN，而是在静态场景中预留一个动态容器，再由系统向该容器添加子实体。

## 静态外壳和动态插槽

示例中的 `static_root` 用 `bsn!` 声明了完整的外层结构：

```text
根节点
└── 面板（静态）
    ├── 标题（静态）
    ├── 说明（静态）
    ├── DynamicContent + Node（静态插槽，开始时没有子项）
    └── StatusLabel（静态）
```

`DynamicContent` 不是用来绘制内容的特殊 UI 类型，而是一个普通的标记组件。它让系统能够准确找到“动态内容应该插入到哪里”，避免依赖实体创建顺序或全局查询所有 `Node`。

## 场景生成之后再填充内容

`commands.spawn_scene(static_root())` 会把 BSN 场景的生成命令加入命令队列。场景真正加入 `World` 后，`Update` 中的系统通过 `Added<DynamicContent>` 找到刚刚出现的插槽：

```rust
fn populate_dynamic_content(
    mut commands: Commands,
    containers: Query<Entity, Added<DynamicContent>>,
    items: Res<DynamicItems>,
) {
    for container in &containers {
        for label in &items.labels {
            commands.spawn((
                Button,
                ChildOf(container),
                Text::new(label.clone()),
            ));
        }
    }
}
```

示例中的实际代码给每个按钮配置了 `Node`、颜色、文字和 `DynamicAction` 数据，并注册了 `Activate` 观察者。关键点是 `ChildOf(container)`：它把新实体挂到 BSN 创建的容器下，因此新按钮会参与该容器的 Flex 布局，同时保留静态面板的边框、标题和状态文本。

## 为什么要在 `Update` 中查询插槽

普通系统中的 `Commands` 是延迟写入 `World` 的。`Startup` 系统只是提交了生成 BSN 场景的命令，不能假定场景的实体已经可以被同一个系统查询。等命令应用后，`Added<DynamicContent>` 会在后续系统执行时匹配到新实体。

如果使用独占系统直接调用 `World` 的生成方法，也可以立即得到实体；但在普通 UI 初始化中，使用“BSN 场景 + `Added` 查询 + `Commands` 添加子实体”的模式更直观。

## 数据变化时如何更新

BSN 描述的是一次场景生成，不会自动监听 `DynamicItems` 的变化。数据改变后，另一个系统需要显式同步 UI，常用做法有两种：

- 数量或顺序变化较大：对动态容器调用 `despawn_children()`，根据最新资源重新生成全部子实体。
- 只修改已有条目的文字、颜色或状态：查询动态条目的组件，直接更新组件字段。

两种方式都不会破坏静态外壳；只更新动态容器的子树即可。若运行时数据在调用 `bsn!` 前就已经准备好，也可以把迭代器表达式传入 `Children`，由 BSN 一次生成一组场景；但数据在场景生成之后才到达时，使用系统追加子实体更合适。

## 职责划分

| 内容 | 推荐实现 |
| --- | --- |
| 面板层级、固定标题、边框、间距、背景 | `bsn!` 静态场景 |
| 来自资源、文件或网络的数据 | `Resource` 或其它 ECS 数据 |
| 根据数据创建、删除、重排实体 | `Commands` 系统 |
| 动态实体的父子关系 | `ChildOf(container)` |
| 动态实体的点击行为 | `observe` 注册 `Activate` 等观察者 |

因此，“静态 UI 使用 BSN、动态 UI 使用 ECS 系统”不是两套互斥方案。把稳定的结构写入 BSN，把变化的部分留出插槽并交给系统管理，就能同时获得静态声明的可读性和动态数据驱动的灵活性。
