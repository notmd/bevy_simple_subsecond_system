use bevy::prelude::*;
use bevy_simple_subsecond_system::prelude::*;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(UiDebugOptions {
            enabled: true,
            ..default()
        })
        .add_plugins(SimpleSubsecondPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, setup.run_if(on_event::<HotPatched>))
        .run();
}

#[hot]
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
}
