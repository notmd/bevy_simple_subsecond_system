use bevy::prelude::*;
use bevy_simple_subsecond_system::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SimpleSubsecondPlugin::default())
        // try adding and removing systems from here! make whole new ones!
        .with_hot_patch(|app: &mut App| {
            // this one won't hotpatch and rerun
            app.add_systems(Startup, setup);
            // this will hot-patch without the #[hot] macro
            // you can change the function signature, and add and remove systems like this at will!
            app.add_systems(Update, do_thing);
            app.add_systems(PostUpdate, do_second_thing);
            // add and remove these! Needs the #[hot(hot_patch_signature = true)] macro to auto-despawn entities spawned in it!
            app.add_systems(StartupRerunHotPatch, spawn_ui);
        })
        .run();
}

#[hot(hot_patch_signature = true)]
fn spawn_ui(mut commands: Commands) {
    commands.queue(|world: &mut World| {
        // Currently bevy forgets to do `track_caller` on `commands.spawn` so to
        // auto-despawn entities spawned inside a StartupRerunHotPatch schedule
        // we need to call spawn on `world` instead.
        world.spawn((
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
            children![Text::new("Bevy is awesome"), Text::new("Try new thing"),],
        ));
    });
}

fn do_thing(res: ResMut<ButtonInput<KeyCode>>) {
    if res.just_pressed(KeyCode::Space) {
        println!("OwO");
    }
}

fn do_second_thing(res: ResMut<ButtonInput<KeyCode>>) {
    if res.just_pressed(KeyCode::Space) {
        println!("UwU");
    }
}

#[derive(Component)]
#[require(Node)]
struct Ui;

fn setup(mut commands: Commands) {
    commands.spawn(Ui);
    commands.spawn(Camera2d);
}
