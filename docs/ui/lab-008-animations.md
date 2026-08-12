# Lab 008：Bevy UI 动画

本实验介绍 Bevy UI 如何实现类似 Web CSS `animation` 和 `transform` 的动态效果。
示例不依赖额外动画库，而是使用 `Time` 和 `Update` 系统每帧修改 UI 组件：

- `UiTransform`：平移、缩放和旋转；
- `BackgroundColor`、`TextColor`：颜色和透明度；
- `Node`：宽度等布局属性。

## 运行示例

```bash
nix develop
just run ui 008
```

## 一、Bevy UI 没有 CSS animation 属性

Web 中可以在 CSS 中直接写：

```css
.card {
  animation: pulse 2s ease-in-out infinite;
  transition: transform 200ms;
}

@keyframes pulse {
  from { transform: translateX(-40px) scale(0.94) rotate(-6deg); }
  to   { transform: translateX(40px) scale(1.06) rotate(6deg); }
}
```

Bevy UI 不解析 CSS，也没有一个等价的 `animation` 或 `transition` 字段。动画本质
上是一个系统在每次 `Update` 运行时，根据经过的时间计算当前值，再写回 Entity 上
对应的 Component：

```rust
fn animate(time: Res<Time>, mut query: Query<&mut UiTransform, With<AnimatedUi>>) {
    let t = time.elapsed_secs();
    for mut transform in &mut query {
        transform.scale = Vec2::splat(1.0 + 0.1 * t.sin());
    }
}
```

`Time::elapsed_secs()` 是从应用启动到当前时刻的累计时间。需要让动画暂停、重置或
只运行一次时，通常再使用一个 Resource 或 Component 保存播放状态、当前进度和
循环次数。

## 二、CSS 常见效果与 Bevy UI 对应关系

| Web CSS 效果 | Bevy UI 的实现方式 | 当前支持情况 |
| --- | --- | --- |
| `translate()` | 修改 `UiTransform::translation`，常用 `Val2::px(x, y)`。 | 可以通过系统平滑实现；不改变 Flex 布局占位。 |
| `scale()` | 修改 `UiTransform::scale`。 | 可以通过系统平滑实现；子实体会随父实体的视觉变换一起变化。 |
| `rotate()` | 修改 `UiTransform::rotation`。 | 可以通过系统平滑实现。 |
| `skew()` | 没有对应的 `UiTransform` 字段。 | 不能直接使用；需要自定义材质或其它渲染方案。 |
| `opacity` | 分别修改 `BackgroundColor`、`BorderColor`、`TextColor`、`ImageNode` 等绘制组件中颜色的 alpha。 | 可以实现，但没有一个自动让整个 UI 子树一起透明的通用 `Opacity` 组件。 |
| 背景/文字/边框颜色过渡 | 每帧重新计算 `Color` 后写回对应组件。 | 可以实现；需要自己选择颜色插值方式和时间曲线。 |
| `width`、`height`、`padding`、`margin`、`gap` | 修改 `Node` 对应字段，通常使用 `Val::Px` 或其它 `Val`。 | 可以实现；每次变化会触发布局重新计算。 |
| `border-radius` | 修改 `Node::border_radius`。 | 可以实现；需要自己计算每帧的圆角值。 |
| `box-shadow` | 修改 `BoxShadow` 和其中的 `ShadowStyle`。 | 可以实现；需要自己更新偏移、模糊、扩散或颜色。 |
| 背景渐变 | 修改 `BackgroundGradient` 中的渐变数据。 | 可以实现；需要自己更新渐变颜色或位置。 |
| `display: none`、`visibility` | 修改 `Display` 或 `Visibility`。 | 可以切换，但属于离散变化，不是连续过渡。 |
| `transform-origin` | 没有 CSS 同名属性。 | 通常通过父子实体拆分、调整布局或额外平移来模拟旋转中心。 |
| `@keyframes`、`ease`、循环、延迟 | 在系统中计算进度，或者使用 `AnimationClip`、`AnimationPlayer`。 | 有实现途径，但没有 CSS 那样的声明式规则。 |

示例中的三个动画分别对应这张表中的三类效果：

1. 第一块同时修改 `translation`、`scale` 和 `rotation`；
2. 第二块分别修改背景和文字的 alpha，说明一个实体的背景透明不会自动影响子实体文字；
3. 第三块修改 `Node::width`，说明布局属性也可以动画化，但会让 UI 布局重新计算。

## 三、`UiTransform`：最接近 CSS transform 的组件

`UiTransform` 只影响布局完成后的视觉变换，不会改变节点在 Flex 或 Grid 中占据的
布局空间：

```rust
UiTransform {
    translation: Val2::px(40, 0),
    scale: Vec2::splat(1.05),
    rotation: Rot2::radians(0.1),
}
```

它的三个字段分别是：

- `translation`：平移，支持 `px`、`percent`、`vw`、`vh` 等 `Val` 表示；
- `scale`：水平和垂直缩放；
- `rotation`：二维旋转。

动画系统只需要在每帧计算这些字段的新值。因为变换发生在布局之后，使用
`UiTransform` 做按钮按下缩放、面板滑入、图标旋转时，不会让同级节点重新排布。

## 四、透明度和颜色动画

Bevy UI 没有 CSS 那种统一的 `opacity` 属性。透明度属于各个绘制组件里的颜色：

```rust
BackgroundColor(Color::srgba(0.2, 0.7, 1.0, alpha));
TextColor(Color::srgba(0.9, 0.95, 1.0, alpha));
```

因此要实现一个包含背景、边框、图片和文字的整体淡入淡出，需要：

1. 给这些绘制实体添加同一个业务标记；
2. 系统根据同一个 `alpha` 分别修改它们的颜色；
3. 对没有颜色 alpha 的离散属性，例如 `Visibility`，在动画开始或结束时单独处理。

这也解释了为什么给父节点的 `BackgroundColor` 设置半透明，并不会自动让子实体的
文字变透明。

## 五、布局属性动画的代价

修改 `Node::width`、`height`、`padding`、`margin` 或 `gap` 类属性，会影响布局计算。
这类动画适合做展开面板、进度条、抽屉宽度和布局尺寸变化，但与只修改
`UiTransform` 相比成本更高，也可能让其它节点随每一帧的布局结果移动。

如果只是想让一个已经布局好的控件移动、缩放或旋转，优先使用 `UiTransform`；如果
确实要改变其它节点的布局关系，再修改 `Node`。

## 六、`AnimationPlayer` 能做什么

Bevy 的通用动画模块提供 `AnimationClip`、`AnimationGraph` 和 `AnimationPlayer`，
可用关键帧曲线驱动实现了 `Animatable` 的组件字段。当前 UI 中，像
`UiTransform::scale`（`Vec2`）和 `UiTransform::rotation`（`Rot2`）这样的字段可以
作为动画曲线目标。

但这不是 CSS 动画的直接替代品：

- 需要创建动画曲线、目标实体和播放组件；
- `UiTransform::translation` 使用 `Val2`，不能直接按普通数值字段插值；
- UI 的 `Color`、`Val`、`Node` 等常用字段不代表都已经有现成的通用曲线类型；
- 对简单的悬浮、按下、淡入淡出和进度变化，手写一个 `Time` 系统通常更直观。

因此本 lab 先使用手写系统展示核心原理。`AnimationPlayer` 更适合多个关键帧、复用
动画片段或复杂播放控制，之后需要时再单独学习。

## 七、Bevy UI 当前能否实现 CSS 常见效果

可以实现的主要是：

- 平移、缩放、旋转；
- 背景、边框、文字和图片的颜色/透明度变化；
- 宽高、内外边距、间距和圆角变化；
- 阴影和渐变参数变化；
- 通过系统实现循环、往返、延迟、播放一次和自定义缓动。

不能直接照搬 CSS 的部分主要是：

- 没有声明式 CSS `animation`、`transition`、`@keyframes` 语法；
- 没有直接的 `skew`、`transform-origin` 和统一 `opacity` 属性；
- `display`、`Visibility` 等显示状态只能离散切换；
- 复杂滤镜、毛玻璃和 `backdrop-filter` 需要自定义材质或渲染实现。

Bevy 的方式更接近“系统驱动组件数据变化”：动画曲线、状态和交互规则由 ECS 系统
控制，UI 组件只保存当前这一帧应该使用的值。
