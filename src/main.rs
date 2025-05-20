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
    use bevy::ecs::system::SystemState;
    let mut system_state: SystemState<(Single<&Greet>, Commands)> = SystemState::new(world);
    let inputs = system_state.get(world);
    bevy_simple_subsecond_system::dioxus_devtools::subsecond::HotFn::current(greet_hotpatched)
        .call(inputs);
    system_state.apply(world);
}

fn greet_hotpatched(greet: Single<&Greet>, mut commands: Commands) {
    info!("{}", greet.0);
}
