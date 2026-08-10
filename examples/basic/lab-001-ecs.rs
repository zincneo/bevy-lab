use bevy::prelude::*;

// 组件只保存数据；这个单元结构体用来标记实体属于本实验。
#[derive(Component, Debug, Default)]
struct DemoComponent;

// Startup 系统在应用启动时执行一次。
fn create_entity(mut commands: Commands) {
    // Commands 会向 World 提交创建实体和添加组件的请求。
    // 请求在当前调度阶段结束时应用到 World，id() 可以立即取得新实体的 ID。
    let entity = commands.spawn(DemoComponent).id();
    println!("创建实体：{entity:?}，组件：{:?}", DemoComponent);
}

// Update 系统在启动系统之后执行；这里查询带有 DemoComponent 的实体。
fn delete_entity(mut commands: Commands, query: Query<(Entity, &DemoComponent)>) {
    for (entity, component) in &query {
        println!("删除实体：{entity:?}，组件：{component:?}");
        // 删除同样通过 Commands 提交，之后由 Bevy 从 World 移除实体及其组件。
        commands.entity(entity).despawn();
    }
}

// Plugin 是一组可复用的 App 配置。最小插件只需要实现 build，注册资源、系统或其他插件。
struct BasicEcsPlugin;

impl Plugin for BasicEcsPlugin {
    fn build(&self, app: &mut App) {
        // Startup 和 Update 是 Bevy 提供的两个基础调度阶段。
        app.add_systems(Startup, create_entity)
            .add_systems(Update, delete_entity);
    }
}

fn main() {
    // App 是 Bevy 应用的容器，内部管理 World、系统和调度信息。
    // add_plugins 会调用 BasicEcsPlugin::build 完成插件注册。
    // 本示例没有窗口插件或循环运行器；默认 runner 只执行一次 app.update()，
    // 因此 Startup 创建实体、Update 删除实体后，程序正常退出。
    App::new().add_plugins(BasicEcsPlugin).run();
}
