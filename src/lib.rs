#![warn(missing_docs)]
#![allow(clippy::type_complexity)]
#![doc = include_str!("../readme.md")]

use __macros_internal::__HotPatchedSystem as HotPatchedSystem;
use __macros_internal::__HotPatchedSystems as HotPatchedSystems;
use bevy::{ecs::system::SystemId, platform::collections::HashMap, prelude::*};
pub use bevy_simple_subsecond_system_macros::*;
pub use dioxus_devtools;
use dioxus_devtools::{subsecond::apply_patch, *};

/// Everything you need to use hotpatching
pub mod prelude {
    pub use super::{DespawnOnHotPatched, HotPatched, SimpleSubsecondPlugin};
    pub use bevy_simple_subsecond_system_macros::*;
}

/// The plugin you need to add to your app:
///
/// ```ignore
/// use bevy::prelude::*;
/// use bevy_simple_subsecond_system::prelude::*;
///
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(SimpleSubsecondPlugin::default())
///     // rest of the setup
///     .run();
/// ```
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct SimpleSubsecondPlugin;

impl Plugin for SimpleSubsecondPlugin {
    fn build(&self, app: &mut App) {
        let (sender, receiver) = crossbeam_channel::bounded::<HotPatched>(1);
        connect(move |msg| {
            if let DevserverMsg::HotReload(hot_reload_msg) = msg {
                if let Some(jumptable) = hot_reload_msg.jump_table {
                    // SAFETY: This is not unsafe, but anything using the updated jump table is.
                    // The table must be built carefully
                    unsafe { apply_patch(jumptable).unwrap() };
                    sender.send(HotPatched).unwrap();
                }
            }
        });

        app.init_resource::<HotPatchedSystems>();

        app.add_event::<HotPatched>().add_systems(
            Last,
            move |mut events: EventWriter<HotPatched>,
                  to_despawn: Query<Entity, With<DespawnOnHotPatched>>,
                  mut commands: Commands| {
                if receiver.try_recv().is_ok() {
                    events.write_default();
                    for entity in to_despawn.iter() {
                        commands.entity(entity).despawn();
                    }
                }
            },
        );
    }
}

/// Event sent in [`Last`] when the hotpatch is applied.
/// Useful to run systems that need to be run after the hotpatch is applied.
///
/// # Example
/// ```ignore
/// # use bevy::prelude::*;
/// # use bevy_simple_subsecond_system::prelude::*;
/// # let mut app = App::new();
///
/// app.add_systems(Startup, setup_ui);
/// app.add_systems(Update, setup_ui.run_if(on_event::<HotPatched>));
///
/// [hot]
/// fn setup_ui(mut commands: Commands) {
///    commands.spawn((
///        DespawnOnHotPatched,
///        Text::new("Hello, world!"),
///    ));
///    commands.spawn((DespawnOnHotPatched, Camera2d));
/// }
/// ```
#[derive(Event, Default)]
pub struct HotPatched;

/// Attach this component to an entity to make it despawn whenever a hotpatch is applied.
/// Useful for spawning things that need to be recreated after a hotpatch.
///
/// # Example
/// ```ignore
/// # use bevy::prelude::*;
/// # use bevy_simple_subsecond_system::prelude::*;
/// # let mut app = App::new();
///
/// app.add_systems(Startup, setup_ui);
/// app.add_systems(Update, setup_ui.run_if(on_event::<HotPatched>));
///
/// [hot]
/// fn setup_ui(mut commands: Commands) {
///    commands.spawn((
///        DespawnOnHotPatched,
///        Text::new("Hello, world!"),
///    ));
///    commands.spawn((DespawnOnHotPatched, Camera2d));
/// }
/// ```
#[derive(Component, Default)]
pub struct DespawnOnHotPatched;

#[doc(hidden)]
pub mod __macros_internal {
    use std::any::TypeId;

    use bevy::{ecs::system::SystemId, platform::collections::HashMap, prelude::*};

    #[derive(Resource, Default)]
    pub struct __HotPatchedSystems(pub HashMap<TypeId, __HotPatchedSystem>);

    #[doc(hidden)]
    pub struct __HotPatchedSystem {
        pub id: SystemId,
        pub current_ptr: u32,
    }
}
