use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Camera2d 让窗口可以显示 UI；UI 元素本身仍然是 World 中的 Entity。
    commands.spawn(Camera2d);

    // children![] 不是把子实体嵌入 Node，而是创建独立的 Entity，
    // 并为它们建立 ChildOf/Children(同样是组件，也就是父子关系是靠实体上持有组件，组件内包含指向其它实体的信息来表达) 关系，组成一个简单的 UI 树。
    commands.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.05, 0.07, 0.12)),
        children![(
            Node {
                width: px(480),
                height: px(260),
                padding: px(24).all(),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(20),
                ..default()
            },
            Text::new("Bevy Content"),
            TextFont::from_font_size(32.0),
            TextColor(Color::srgb(0.9, 0.95, 1.0)),
            BackgroundColor(Color::srgb(0.12, 0.18, 0.28)),
            children![
                (
                    Text::new("Bevy UI"),
                    TextFont::from_font_size(32.0),
                    TextColor(Color::srgb(0.9, 0.95, 1.0)),
                ),
                (
                    ImageNode::solid_color(Color::srgb(0.2, 0.65, 0.85)),
                    Node {
                        width: px(180),
                        height: px(72),
                        ..default()
                    },
                ),
            ],
        )],
    ));
}
