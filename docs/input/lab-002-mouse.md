# Lab 002：鼠标输入

Bevy 将鼠标按钮状态放在 `ButtonInput<MouseButton>` 资源中。鼠标移动和滚轮则分别通过 `AccumulatedMouseMotion` 与 `AccumulatedMouseScroll` 提供每个更新周期的累计值。

## 鼠标按钮

`ButtonInput<MouseButton>` 的查询方式与键盘相同：

- `just_pressed(MouseButton::Left)`：左键在当前周期刚按下；
- `pressed(MouseButton::Left)`：左键当前仍处于按下状态；
- `just_released(MouseButton::Left)`：左键在当前周期刚释放。

`MouseButton` 常用值包括 `Left`、`Right` 和 `Middle`。

## 移动和滚轮

`AccumulatedMouseMotion.delta` 是当前周期收到的所有鼠标移动量之和，类型为 `Vec2`。该资源每个周期都会重置为零，因此系统应在需要时直接读取本周期的值。

`AccumulatedMouseScroll.delta` 是当前周期收到的所有滚轮变化量之和，`unit` 表示它使用行（`Line`）还是像素（`Pixel`）作为单位。示例只打印原始单位和值；实际使用时应根据自己的缩放或滚动逻辑处理它们。

## 示例如何运行

```bash
nix develop
just run input 002
```

在打开的窗口中点击或按住左键、移动鼠标、滚动滚轮，结果会打印到终端。按 `Escape` 退出。示例依赖窗口插件，因此需要图形环境。
