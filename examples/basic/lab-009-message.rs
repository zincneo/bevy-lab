use bevy::prelude::*;

#[derive(Message, Debug)]
struct Notice(&'static str);

#[derive(Message, Debug, Default)]
struct Ping;

fn send_startup_messages(mut writer: MessageWriter<Notice>) {
    writer.write(Notice("Startup 发送的第一条消息"));
    writer.write_batch([
        Notice("批量发送的第一条消息"),
        Notice("批量发送的第二条消息"),
    ]);
}

fn read_notices(mut reader: MessageReader<Notice>) {
    for notice in reader.read() {
        println!("MessageReader：{}", notice.0);
    }
}

fn write_ping(mut writer: MessageWriter<Ping>) {
    writer.write_default();
}

fn read_pings(mut reader: PopulatedMessageReader<Ping>) {
    for ping in reader.read() {
        println!("PopulatedMessageReader：收到 {:?}", ping);
    }
}

fn process_ping() {
    println!("on_message：至少有一条 Ping 可以处理");
}

fn main() {
    App::new()
        .add_message::<Notice>()
        .add_message::<Ping>()
        .add_systems(Startup, send_startup_messages)
        .add_systems(
            Update,
            (
                read_notices,
                write_ping,
                read_pings,
                process_ping.run_if(on_message::<Ping>),
            )
                .chain(),
        )
        .run();
}
