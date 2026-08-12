# bevy-lab Agent Guide

## 项目目标

本项目用于跟随 GitHub 上 Bevy `main` 分支学习和验证 Bevy 特性。每个 lab 应保持单一主题、可独立运行；较长的原理说明放在 `docs/`，不要把大段教程注释放进示例源码。

## 开发环境

本项目使用 Nix Flake 提供 Rust、Bevy 图形运行时和系统库环境。运行 `cargo`、`just` 或图形示例前，必须先进入开发环境：

```bash
nix develop
```

进入该 shell 后再执行命令。Agent 不能保持交互式 shell 时，使用等价形式：

```bash
nix develop -c just catalog
nix develop -c cargo check
```

不要假设宿主环境中已经有完整的 Bevy 编译和运行依赖。

## 文件结构与命名

每个主题是 `examples/` 下的一级目录，目录名使用简短的主题名，例如 `2d`、`3d`、`ecs`、`rendering`。

```text
examples/<topic>/topic
examples/<topic>/lab-<NNN>-<kebab-case-name>.rs
docs/<topic>/lab-<NNN>-<kebab-case-name>.md
```

规则：

1. 每个主题目录必须存在无扩展名的 `topic` 文件。
2. `topic` 第一行是主题简介；空一行后，每行使用 `NNN name 描述` 记录该编号 lab 的演示内容，其中 `name` 必须与 `lab-NNN-name.rs` 和对应 `.md` 文件名一致。
3. lab 文件名固定为 `lab-NNN-name.rs`，编号在同一主题内三位补零且唯一，通常按递增顺序创建。
4. 可运行 lab 必须同时有同名的 `examples/<topic>/...rs` 和 `docs/<topic>/...md`。纯流程、平台打包等无法单独运行的专题可以只有 `docs/<topic>/...md`，并且不注册 Cargo example。
5. 文档文件用于完整解释、设计背景和运行观察；源码注释只保留必要上下文。

## Cargo 注册

新增可运行 `.rs` 示例后，必须在根目录 `Cargo.toml` 增加对应的 `[[example]]`，target 名称为 `<topic>-lab-NNN-name`。纯文档专题不增加 `[[example]]`：

```toml
[[example]]
name = "2d-lab-001-light"
path = "examples/2d/lab-001-light.rs"
```

同时检查 `name`、`path`、`topic` 条目和 `docs` 文件的编号与名称一致。纯文档专题只检查 `topic` 和 `docs` 文件的编号与名称一致。

## Just 命令

所有命令都在 `nix develop` 环境中运行：

```bash
just catalog                         # 主题目录名和 topic 第一行简介
just list <topic>                    # 原样打印该主题的 topic 文件
just run <topic> <NNN>               # 运行对应 lab
just run <topic> <NNN> -- --help     # 向 lab 传递参数
```

修改目录或命名后，至少执行 `just catalog`、`just list <topic>`，并在有可运行示例时执行 `just run <topic> <NNN>` 验证。

## Agent 工作要求

- 开始任何构建、运行或图形相关检查前，确认使用了 `nix develop`。
- 新增 lab 时同步更新 `topic`、`Cargo.toml`、`examples/` 和 `docs/`，不要只添加其中一项。
- 保留现有实验；不要为了整理目录删除或重命名已有 lab，除非用户明确要求。
- 修改 `justfile` 后运行 `just --fmt --check`，并检查命令的错误提示和参数校验。
