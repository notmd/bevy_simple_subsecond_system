use bevy::prelude::*;
use bevy_simple_subsecond_system::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SimpleSubsecondPlugin::default())
        .add_event::<MyEvent>()
        .add_systems(Startup, setup)
        .add_hot_plugin(|app: &mut App| {
            app.add_systems(Update, some_reader);
            app.add_systems(Update, write_event);
            app.add_systems(Update, configure_ui);
        })
        .run();
}

#[derive(Component)]
#[require(Node)]
struct Ui;

fn setup(mut commands: Commands) {
    commands.spawn(Ui);
    commands.spawn(Camera2d);
}

#[derive(Event)]
pub struct MyEvent;

fn write_event(mut event_writer: EventWriter<MyEvent>, key: Res<ButtonInput<KeyCode>>) {
    if key.just_pressed(KeyCode::Space) {
        println!("Writing");
        event_writer.write(MyEvent);
    }
}

fn some_reader(mut event_reader: EventReader<MyEvent>) {
    for _ in event_reader.read() {
        println!("read eventtttttttt");
    }
}

#[hot]
fn configure_ui(ui: Single<Entity, With<Ui>>, mut commands: Commands) {
    commands.entity(*ui).despawn_related::<Children>().insert((
        Node {
            // You can change the `Node` however you want at runtime
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        },
        children![
            Text::new("Hello, worldaa!"),
            Text::new("Try adding new texts below awa"),
        ],
    ));
}
