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
    pub use super::{HotPatched, HotPatchedApp, HotPatchedAppExt as _, SimpleSubsecondPlugin};
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

/// Wrapper around [`App`] used by [`HotPatchedAppExt::with_hot_patch`], which allows you to add and remove systems at runtime.
#[derive(Deref, DerefMut)]
pub struct HotPatchedApp(send_wrapper::SendWrapper<bevy::app::App>);

impl Default for HotPatchedApp {
    fn default() -> Self {
        HotPatchedApp(send_wrapper::SendWrapper::new(bevy::app::App::default()))
    }
}

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
struct HotPatchUpdate;

/// Trait for [`App`] to add and remove systems at runtime.
pub trait HotPatchedAppExt {
    /// Call this with plugins and systems and it will auto-add and remove systems in the `Update` schedule to your running app.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use bevy::prelude::*;
    /// # use bevy_simple_subsecond_system::prelude::*;
    ///
    /// App::new()
    ///     .add_plugins(DefaultPlugins)
    ///     .add_plugins(SimpleSubsecondPlugin::default())
    ///     .with_hot_patch(|mut app| {
    ///         app.add_systems(Update, my_system);
    ///         app
    ///     });
    ///
    /// fn my_system() {
    ///     info!("Hello, world!");
    /// }
    /// ```
    fn with_hot_patch(
        &mut self,
        func: impl FnMut(HotPatchedApp) -> HotPatchedApp + Send + Sync + 'static,
    ) -> &mut App;
}

impl HotPatchedAppExt for App {
    fn with_hot_patch(
        &mut self,
        mut func: impl FnMut(HotPatchedApp) -> HotPatchedApp + Send + Sync + 'static,
    ) -> &mut App {
        // we run this once during startup here so that way when we are actually restarting the app all the systems get added
        let mut reload_app = func(HotPatchedApp::default());
        if let Some(mut schedules) = reload_app.world_mut().get_resource_mut::<Schedules>() {
            if let Some(mut update) = schedules.remove(Update) {
                let hot_reload_update = schedules.entry(HotPatchUpdate);
                *hot_reload_update.graph_mut() = std::mem::take(update.graph_mut());
                let result = hot_reload_update.initialize(self.world_mut());
                if let Err(e) = result {
                    error!("Failed to initialize hotpatch update: {e}");
                    return self;
                }
            }
        }
        self.add_systems(Update, |world: &mut World| {
            let _ = world.try_run_schedule(HotPatchUpdate);
        });
        let reloadable_section =
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
                let reload_app = reloadable_section
                    .lock()
                    .unwrap()
                    .try_call((HotPatchedApp::default(),));
                let mut reload_app = match reload_app {
                    Ok(reload_app) => reload_app,
                    Err(e) => {
                        error!("Failed to call hotpatch function: {e:?}");
                        return;
                    }
                };
                let Some(mut reload_schedules) =
                    reload_app.world_mut().get_resource_mut::<Schedules>()
                else {
                    return;
                };

                let Some(mut reload_update) = reload_schedules.remove(Update) else {
                    return;
                };
                schedules.remove(HotPatchUpdate);
                let hot_reload_update = schedules.entry(HotPatchUpdate);
                *hot_reload_update.graph_mut() = std::mem::take(reload_update.graph_mut());
                commands.run_system_cached(|world: &mut World| {
                    world.schedule_scope(HotPatchUpdate, |world, hot_reload_update| {
                        let result = hot_reload_update.initialize(world);
                        if let Err(e) = result {
                            error!("Failed to initialize hotpatch update: {e}");
                        }
                    });
                });
            },
        );
        self
    }
}
