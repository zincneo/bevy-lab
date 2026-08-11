# Lab 003：手柄输入

手柄和键盘、鼠标不同：一个应用可能同时连接多个手柄，因此 Bevy 不把所有手柄状态放在一个全局 `Resource` 中。每个已连接的手柄对应一个带有 `Gamepad` 组件的 Entity，系统通过 `Query` 遍历它们。

## 读取手柄状态

`Gamepad` 提供了几类常用查询：

- `just_pressed(GamepadButton::South)`：某个数字按键在当前周期刚刚按下；
- `pressed(button)`：按键当前仍处于按下状态；
- `just_released(button)`：按键在当前周期刚刚释放；
- `get(GamepadAxis::LeftStickX)`：读取摇杆轴或模拟扳机的数值。

模拟输入通常是 `-1.0..=1.0` 的轴值，扳机等输入通常是 `0.0..=1.0`。实际控制逻辑一般还会设置死区，避免摇杆轻微漂移被当成有效输入。

## 连接和断开

手柄连接状态变化通过 `GamepadConnectionEvent` 消息通知。需要知道设备何时加入或离开时，使用 `MessageReader<GamepadConnectionEvent>`；需要持续读取当前状态时，查询 `Gamepad` Entity。

示例同时演示了这两种方式：连接/断开使用消息，按钮和轴使用 `Query`。

## 示例如何运行

```bash
nix develop
just run input 003
```

连接手柄后按下 South 按键、推动左摇杆或按压右扳机，结果会打印到终端。按 `Escape` 退出。没有连接手柄时，查询结果为空，但程序仍然可以正常启动。
