use bevy::prelude::*;
use bevy_simple_subsecond_system::prelude::*;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SimpleSubsecondPlugin::default())
        .add_systems(Startup, setup)
        .run();
}

#[hot(rerun_on_hot_patch = true)]
fn setup(mut commands: Commands) {
    commands.spawn((
        DespawnOnHotPatched,
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
            Text::new("Hello, world!"),
            Text::new("Try adding new texts below"),
        ],
    ));
    commands.spawn((DespawnOnHotPatched, Camera2d));

    commands.insert_resource(UiDebugOptions {
        // Set this to `true` to see the UI debug overlay. Try changing it at runtime!
        enabled: false,
        ..default()
    });
}
