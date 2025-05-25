use bevy::prelude::*;
use bevy_simple_subsecond_system::prelude::*;
fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SimpleSubsecondPlugin::default())
        .add_systems(Update, trigger_greeting)
        .add_observer(greet)
        .run()
}

#[hot]
fn trigger_greeting(mut commands: Commands) {
    commands.trigger(PrintGreeting);
}

#[hot]
fn greet(_trigger: Trigger<PrintGreeting>) {
    info_once!(
        "Hello from a hotpatched observer! Try changing this string while the app is running!"
    );
}

#[derive(Event)]
struct PrintGreeting;
