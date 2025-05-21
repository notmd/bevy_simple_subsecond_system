#![warn(missing_docs)]
#![allow(clippy::type_complexity)]
#![doc = include_str!("../readme.md")]

use __macros_internal::__HotPatchedSystems as HotPatchedSystems;
use bevy::prelude::*;
pub use bevy_simple_subsecond_system_macros::*;
pub use dioxus_devtools;
use dioxus_devtools::{subsecond::apply_patch, *};

/// Everything you need to use hotpatching
pub mod prelude {
    pub use super::{HotPatched, SimpleSubsecondPlugin};
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
            PreUpdate,
            (
                move |mut events: EventWriter<HotPatched>| {
                    if receiver.try_recv().is_ok() {
                        events.write_default();
                    }
                },
                update_system_ptr,
            )
                .chain(),
        );
    }
}

/// Event sent when the hotpatch is applied.
#[derive(Event, Default)]
pub struct HotPatched;

fn update_system_ptr(hot_patched_systems: Res<HotPatchedSystems>, mut commands: Commands) {
    for system in hot_patched_systems.0.values() {
        commands.run_system(system.system_ptr_update_id);
    }
}
#[doc(hidden)]
pub mod __macros_internal {
    use std::any::TypeId;

    use bevy::{ecs::system::SystemId, platform::collections::HashMap, prelude::*};

    #[derive(Resource, Default)]
    pub struct __HotPatchedSystems(pub HashMap<TypeId, __HotPatchedSystem>);

    #[doc(hidden)]
    pub struct __HotPatchedSystem {
        pub system_ptr_update_id: SystemId,
        pub current_ptr: u64,
        pub last_ptr: u64,
    }
}
