# bevy-lab

这个仓库用于长期跟踪 GitHub 上 Bevy `main` 分支，并把学习、验证与试验拆分成可独立运行的小型 lab。

## 开发环境

项目使用 Nix Flake 提供 Bevy 所需的 Rust、图形运行时和系统库。运行 `cargo`、`just` 或示例前，先进入开发环境：

```bash
nix develop
```

非交互场景可直接使用 `nix develop -c <命令>`，例如：

```bash
nix develop -c just catalog
```

## 目录约定

```text
examples/
  2d/
    topic
    lab-001-light.rs
docs/
  2d/
    lab-001-light.md
```

- `examples/<主题>/` 是一级主题目录，例如 `2d`、`3d`、`ecs`、`rendering`。
- 每个主题目录必须有一个无扩展名的 `topic` 文件：第一行是主题简介；空一行后，每行以 `三位编号 空格 描述` 记录对应 lab 的演示内容。例如：

  ```text
  2D 渲染、精灵与光照相关实验。

  001 演示 2D 光照对精灵材质的影响。
  ```

- 每个示例固定命名为 `lab-<三位编号>-<kebab-case-主题>.rs`，如 `lab-001-light.rs`。编号在同一主题内唯一，并按编号递增。
- 每个 lab 都有一组同名文件：`examples/<主题>/lab-<编号>-<名称>.rs` 存放可运行代码，`docs/<主题>/lab-<编号>-<名称>.md` 存放说明文档。

## 注册 lab

新增 lab 时，创建对应的 `.rs`、`.md` 和 `topic` 条目，并在 `Cargo.toml` 中注册 example target。target 名称统一为 `<主题>-lab-<编号>-<名称>`：

```toml
[[example]]
name = "2d-lab-001-light"
path = "examples/2d/lab-001-light.rs"
```

## Just 命令

`justfile` 提供三个面向日常浏览与运行的命令：

```bash
# 查看所有主题目录
just catalog

# 查看 2d 主题的完整说明与 lab 清单
just list 2d

# 运行 2d 的第 001 个 lab
just run 2d 001

# 将额外参数传给 lab 程序
just run 2d 001 -- --help
```

| 命令 | 用途 |
| --- | --- |
| `just` | 显示可用命令。 |
| `just catalog` | 显示所有主题；目录名为蓝色加粗，内容取自各自 `topic` 的第一行。 |
| `just list <主题>` | 原样打印该主题的 `topic` 文件。 |
| `just run <主题> <三位编号> [参数...]` | 执行对应的 `cargo run --example`，并将参数传给 lab 程序。 |

执行 `just` 可查看全部可用命令和简要帮助。
