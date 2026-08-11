# Lab 013：Plugin

## 学习目标

了解 Plugin 如何把一组资源、System 和其他 App 配置封装起来，并通过 `add_plugins` 安装到 App 中。

## Plugin 是什么

Plugin 不是一个会被循环调用的 System，而是一段 App 配置逻辑。实现 `Plugin` 特征后，可以在 `build` 方法中向 App 注册资源、System、Schedule 和其他插件。

```rust
struct GreetingPlugin;

impl Plugin for GreetingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PluginState>()
            .add_systems(Update, update_from_plugin);
    }
}
```

`build` 在插件加入 App 时执行一次。之后真正反复运行的是插件注册的 System。

## 添加自定义 Plugin

使用 `App::add_plugins` 添加插件：

```rust
App::new()
    .add_plugins(MinimalPlugins)
    .add_plugins(GreetingPlugin)
    .run();
```

插件通常在 `build` 中完成三类工作：

- 初始化或插入插件需要的 Resource；
- 把 System 注册到合适的 Schedule；
- 设置插件相关的 App 配置。

应用的 `main` 函数只需要安装插件，不必重复知道插件内部注册了哪些 System。

## Plugin 与普通函数的区别

普通 System 描述“运行时要执行的行为”：

```rust
app.add_systems(Update, update_score);
```

Plugin 描述“如何把一组功能接入 App”：

```rust
app.add_plugins(ScorePlugin);
```

一个 Plugin 可以注册多个 Resource 和 System，也可以组合其他 Plugin。这样可以把功能按模块拆分，保持 `main` 函数简洁。

## 示例流程

本示例的 `GreetingPlugin` 会：

1. 初始化 `PluginRuns` Resource；
2. 注册一个 `Update` System；
3. 每次更新打印运行次数；
4. 运行三次后通过 `AppExit` 退出。

```text
App 添加 GreetingPlugin
        ↓
Plugin::build 注册 Resource 和 System
        ↓
App 运行时执行插件注册的 System
```

## 常见用法

把某个独立功能需要的 Resource、System 和配置放进一个 Plugin，是 Bevy 中组织代码的常见方式。窗口、渲染、音频等大型功能也通常通过插件提供；本 lab 只演示自定义 Plugin 的最小用法。
