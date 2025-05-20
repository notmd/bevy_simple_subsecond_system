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
struct Ui {
    count: f32,
    count2: f32,
    count3: f32,
}

fn setup(mut commands: Commands) {
    commands.spawn(Ui {
        count: 0.0,
        count2: 0.0,
        count3: 0.0,
    });
    commands.spawn(Camera2d);
}

#[hot]
fn configure_ui(ui: Single<(Entity, &Ui)>, mut commands: Commands) {
    let (entity, ui) = ui.into_inner();
    commands
        .entity(entity)
        .despawn_related::<Children>()
        .insert((
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
}
