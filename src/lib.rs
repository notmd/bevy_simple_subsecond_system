#![warn(missing_docs)]
#![allow(clippy::type_complexity)]
#![doc = include_str!("../readme.md")]

#[cfg(all(not(target_family = "wasm"), debug_assertions))]
use __macros_internal::__HotPatchedSystems as HotPatchedSystems;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
pub use bevy_simple_subsecond_system_macros::*;
pub use dioxus_devtools;
#[cfg(all(not(target_family = "wasm"), debug_assertions))]
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

#[derive(Deref, DerefMut)]
pub struct HotPatchedApp(send_wrapper::SendWrapper<bevy::app::App>);
impl HotPatchedApp {
    pub fn new() -> HotPatchedApp {
        HotPatchedApp(send_wrapper::SendWrapper::new(bevy::app::App::default()))
    }
}
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct HotReloadUpdate;

pub trait ReloadableAppExt {
    /// Call this with plugins and systems and it will auto-add and remove systems in the `Update` schedule to your running app
    fn reloadable(
        &mut self,
        func: impl FnMut(HotPatchedApp) -> HotPatchedApp + Send + Sync + 'static,
    ) -> &mut App;
}

impl ReloadableAppExt for App {
    fn reloadable(
        &mut self,
        mut func: impl FnMut(HotPatchedApp) -> HotPatchedApp + Send + Sync + 'static,
    ) -> &mut App {
        // we run this once during startup here so that way when we are actually restarting the app all the systems get added
        let mut reload_app = func(HotPatchedApp::new());
        if let Some(mut schedules) = reload_app.world_mut().get_resource_mut::<Schedules>() {
            if let Some(mut update) = schedules.remove(Update) {
                let mut hot_reload_update = schedules.entry(HotReloadUpdate);
                *hot_reload_update.graph_mut() = std::mem::take(update.graph_mut());
                hot_reload_update.initialize(self.world_mut()).unwrap();
            }
        }
        self.add_systems(Update, |world: &mut World| {
            let _ = world.try_run_schedule(HotReloadUpdate);
        });
        let mut reloadable_section =
            std::sync::Mutex::new(dioxus_devtools::subsecond::HotFn::current(func));
        self.add_systems(
            PostUpdate,
            move |_: Option<NonSend<NonSendMarker>>,
                  mut schedules: ResMut<Schedules>,
                  mut commands: Commands,
                  hotreload_event: EventReader<HotPatched>| {
                if hotreload_event.is_empty() {
                    return;
                }
                let mut reload_app = reloadable_section
                    .lock()
                    .unwrap()
                    .try_call((HotPatchedApp::new(),))
                    .unwrap();
                let Some(mut reload_schedules) =
                    reload_app.world_mut().get_resource_mut::<Schedules>()
                else {
                    return;
                };

                let Some(mut reload_update) = reload_schedules.remove(Update) else {
                    return;
                };
                schedules.remove(HotReloadUpdate);
                let mut hot_reload_update = schedules.entry(HotReloadUpdate);
                *hot_reload_update.graph_mut() = std::mem::take(reload_update.graph_mut());
                commands.run_system_cached(|world: &mut World| {
                    world.schedule_scope(HotReloadUpdate, |world, hot_reload_update| {
                        hot_reload_update.initialize(world).unwrap();
                    });
                });
            },
        );
        self
    }
}
