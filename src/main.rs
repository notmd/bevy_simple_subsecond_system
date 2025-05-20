use bevy::prelude::*;
use bevy_simple_subsecond_system::prelude::*;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SimpleSubsecondPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, greet)
        .run();
}

#[derive(Component)]
struct Greet(String);

fn setup(mut commands: Commands) {
    commands.spawn(Greet("Hello".to_string()));
}

fn greet(world: &mut World) {
    bevy_simple_subsecond_system::dioxus_devtools::subsecond::HotFn::current(
        greet_hotpatched_exclusive,
    )
    .call((world,))
    .unwrap();
}

fn greet_hotpatched_exclusive(
    world: &mut bevy::ecs::world::World,
) -> std::result::Result<(), bevy::ecs::system::RegisteredSystemError> {
    world.run_system_cached(greet_hotpatched_original)
}

fn greet_hotpatched_original(greet: Single<&Greet>, mut commands: Commands) {
    info!("{}", greet.0);
}
