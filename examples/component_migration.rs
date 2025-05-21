use bevy::prelude::*;
use bevy_simple_subsecond_system::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SimpleSubsecondPlugin::default())
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn((
                Example {
                    field_a: 5,
                    ..Default::default()
                },
                Example2 {
                    field_b: 1005,
                    ..Default::default()
                },
            ));
            commands.spawn((
                Example {
                    field_a: 8,
                    ..Default::default()
                },
                Example2 {
                    field_b: 1008,
                    ..Default::default()
                },
            ));
        })
        .add_systems(Startup, register_components)
        .add_systems(Update, print_components)
        .insert_resource(PrintTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .run();
}

#[derive(Resource)]
struct PrintTimer(Timer);

// This is #[hot] because new versions of hot patched components need
// to be registered so their reflected type data are available
#[hot(rerun_on_hot_patch = true)]
fn register_components(registry: Res<AppTypeRegistry>) {
    let mut registry = registry.write();
    registry.register::<Example>();
    registry.register::<Example2>();
}

// Try changing the components below:
// - Rename them
// - Add a field
// - Remove a field

#[derive(Debug, Reflect, Component, Default, HotPatchMigrate)]
#[reflect(Component, Default, HotPatchMigrate)]
struct Example {
    field_a: usize,
}

#[derive(Debug, Reflect, Component, Default, HotPatchMigrate)]
#[reflect(Component, Default, HotPatchMigrate)]
struct Example2 {
    field_b: usize,
}

#[hot]
fn print_components(
    time: Res<Time>,
    mut timer: ResMut<PrintTimer>,
    q: Query<(&Example, &Example2)>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for v in q {
            info!("{v:?}");
        }
    }
}
