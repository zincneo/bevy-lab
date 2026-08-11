# Lab 001：键盘输入

Bevy 的键盘输入由 `DefaultPlugins` 中的窗口和输入插件接收，再更新为系统可以读取的 `ButtonInput<KeyCode>` 资源。系统不需要直接读取操作系统事件，只需要在参数中请求这个资源。

## `ButtonInput<KeyCode>` 的三种常用查询

- `just_pressed(key)`：这个更新周期刚刚按下时为 `true`，只持续一个周期，适合触发一次性动作。
- `pressed(key)`：按键保持按下时持续为 `true`，适合持续移动或蓄力。
- `just_released(key)`：这个更新周期刚刚释放时为 `true`，同样只持续一个周期。

`KeyCode` 表示键在键盘上的物理位置，例如 `KeyCode::KeyA`、`KeyCode::ArrowUp` 和 `KeyCode::Space`。用它做控制绑定时，不会因为键盘布局改变而改变按键位置。文本输入（字符、输入法等）属于另一类输入，后续再单独学习。

## 示例如何运行

```bash
nix develop
just run input 001
```

示例会打开一个 Bevy 窗口，同时将输入结果打印到启动命令的终端：

- 按下并释放 `Space`，观察 `just pressed`；
- 按住 `ArrowUp`，观察持续输出的 `pressed`；
- 松开 `ArrowUp`，观察 `just released`；
- 按 `Escape` 发送 `AppExit::Success`，结束程序。

这个示例依赖窗口插件接收真实的键盘输入，因此需要图形环境；仅有纯终端或无显示设备的环境不能产生真实按键输入。
