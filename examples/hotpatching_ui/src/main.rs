use bevy::prelude::*;
use bevy_simple_subsecond_system::prelude::*;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SimpleSubsecondPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, configure_ui)
        .run();
}

#[derive(Component)]
#[require(Node)]
struct Ui;

fn setup(mut commands: Commands) {
    commands.spawn(Ui);
    commands.spawn(Camera2d);
}

#[hot]
fn configure_ui(ui: Single<Entity, With<Ui>>, mut commands: Commands) {
    commands.entity(*ui).despawn_related::<Children>().insert((
        Node {
            flex_direction: FlexDirection::Row,
            ..default()
        },
        children![
            Text::new("Hello, world!"),
            Text::new("Here's a little demo"),
            Text::new("I can add new texts!"),
            Text::new("I can add new texts!"),
            Text::new("I can add new texts!"),
            Text::new("I can add new texts!"),
            Text::new("I can change existing texts!"),
        ],
    ));
}
