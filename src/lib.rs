#![warn(missing_docs)]
#![allow(clippy::type_complexity)]
#![doc = include_str!("../readme.md")]

use bevy::prelude::*;
pub use bevy_simple_subsecond_system_macros::*;
pub use dioxus_devtools;

/// Everything you need to use hotpatching
pub mod prelude {
    pub use super::SimpleSubsecondPlugin;
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
    fn build(&self, _app: &mut App) {
        dioxus_devtools::connect_subsecond();
    }
}
