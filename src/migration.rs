//! TODO

use bevy::{
    ecs::{
        entity::Entity,
        reflect::{AppTypeRegistry, ReflectComponent},
        resource::Resource,
        system::{Commands, Query, Res, ResMut, RunSystemOnce},
        world::World,
    },
    platform::collections::HashMap,
    prelude::{Deref, DerefMut},
    reflect::{FromType, Reflect, prelude::ReflectDefault},
};
use std::{
    any::{Any, TypeId},
    sync::Arc,
};

/// TODO
pub trait HotPatchMigrate: Any + Reflect + Default {
    /// TODO
    fn current_type_id() -> TypeId;
}

/// TODO
#[derive(Clone)]
pub struct ReflectHotPatchMigrate(pub Arc<dyn Fn() -> TypeId + Sync + Send + 'static>);

impl<T: HotPatchMigrate> FromType<T> for ReflectHotPatchMigrate {
    fn from_type() -> Self {
        Self(Arc::new(T::current_type_id))
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub(crate) struct ComponentMigrations(
    HashMap<TypeId, Arc<dyn Fn() -> TypeId + Sync + Send + 'static>>,
);

pub(crate) fn migrate(world: &mut World) {
    world
        .run_system_cached(register_migratable_components)
        .unwrap();

    let migrations = world.resource::<ComponentMigrations>();
    let changed: Vec<_> = migrations
        .iter()
        .filter(|(prev, current)| prev != &&current())
        .map(|(prev, current)| (prev.clone(), current.clone()))
        .collect();

    for (prev, current) in &changed {
        migrate_component(world, prev.clone(), current());
    }

    // Track hot patches to the new struct
    let mut migrations = world.resource_mut::<ComponentMigrations>();
    migrations.extend(changed.into_iter().map(|(_, current)| (current(), current)));
}

fn register_migratable_components(
    mut migrations: ResMut<ComponentMigrations>,
    registry: Res<AppTypeRegistry>,
) {
    for registration in registry.read().iter() {
        let Some(current_type_id) = registration.data::<ReflectHotPatchMigrate>() else {
            continue;
        };

        migrations
            .entry(registration.type_id())
            .or_insert_with(|| current_type_id.clone().0);
    }
}

fn migrate_component(world: &mut World, from: TypeId, to: TypeId) {
    world
        .run_system_once(
            move |es: Query<Entity>, world: &World, mut commands: Commands| {
                for e in es {
                    match world.get_reflect(e, from) {
                        Ok(c) => {
                            let c = c.reflect_clone().unwrap();
                            commands.queue(move |world: &mut World| {
                                world.resource_scope::<AppTypeRegistry, ()>(|world, registry| {
                                    let registry = registry.read();
                                    let reflect_default =
                                        registry.get_type_data::<ReflectDefault>(to).unwrap();
                                    let reflect_component =
                                        registry.get_type_data::<ReflectComponent>(to).unwrap();

                                    let entity = &mut world.entity_mut(e);

                                    reflect_component.insert(
                                        entity,
                                        &*reflect_default.default(),
                                        &registry,
                                    );

                                    reflect_component.apply(entity, &*c);
                                });
                            });
                        }
                        Err(_) => {}
                    }
                }
            },
        )
        .unwrap();
}
