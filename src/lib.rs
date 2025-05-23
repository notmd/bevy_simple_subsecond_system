#![warn(missing_docs)]
#![allow(clippy::type_complexity)]
#![doc = include_str!("../readme.md")]

pub mod migration;

#[cfg(all(not(target_family = "wasm"), debug_assertions))]
use __macros_internal::__HotPatchedSystems as HotPatchedSystems;
use bevy_app::{App, Plugin, PostStartup, PreUpdate};
#[cfg(all(not(target_family = "wasm"), debug_assertions))]
use bevy_ecs::system::{Commands, Res};
use bevy_ecs::{
    event::{Event, EventWriter},
    schedule::IntoScheduleConfigs,
};
pub use bevy_simple_subsecond_system_macros::*;
pub use dioxus_devtools;
#[cfg(all(not(target_family = "wasm"), debug_assertions))]
use dioxus_devtools::{subsecond::apply_patch, *};

/// Everything you need to use hotpatching
pub mod prelude {
    pub use super::{HotPatched, SimpleSubsecondPlugin};
    pub use crate::migration::*;
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
        #[cfg(target_family = "wasm")]
        {
            let _ = app;
            warn!("Hotpatching is not supported on Wasm yet. Disabling SimpleSubsecondPlugin.");
            return;
        }
        #[cfg(not(debug_assertions))]
        {
            return;
        }
        #[cfg(all(not(target_family = "wasm"), debug_assertions))]
        {
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
            app.init_resource::<migration::ComponentMigrations>();

            app.add_event::<HotPatched>()
                .add_systems(
                    PreUpdate,
                    (
                        update_system_ptr,
                        move |mut events: EventWriter<HotPatched>, mut commands: Commands| {
                            if receiver.try_recv().is_ok() {
                                events.write_default();
                                commands.run_system_cached(migration::migrate);
                            }
                        },
                    )
                        .chain(),
                )
                .add_systems(PostStartup, migration::register_migratable_components);
        }
    }
}

/// Event sent when the hotpatch is applied.
#[derive(Event, Default)]
pub struct HotPatched;

#[cfg(all(not(target_family = "wasm"), debug_assertions))]
fn update_system_ptr(hot_patched_systems: Res<HotPatchedSystems>, mut commands: Commands) {
    for system in hot_patched_systems.0.values() {
        commands.run_system(system.system_ptr_update_id);
    }
}
#[doc(hidden)]
pub mod __macros_internal {
    pub use bevy_ecs::{
        system::{IntoSystem, SystemId, SystemState},
        world::World,
    };
    pub use bevy_ecs_macros::Resource;
    pub use bevy_log::debug;
    use bevy_platform::collections::HashMap;
    use std::any::TypeId;

    #[derive(Resource, Default)]
    pub struct __HotPatchedSystems(pub HashMap<TypeId, __HotPatchedSystem>);

    #[doc(hidden)]
    pub struct __HotPatchedSystem {
        pub system_ptr_update_id: SystemId,
        pub current_ptr: u64,
        pub last_ptr: u64,
    }
}
