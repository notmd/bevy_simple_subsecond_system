#![allow(unused_mut, unused_variables)]
use bevy::prelude::*;
use bevy_simple_subsecond_system::prelude::*;

#[hot]
fn empty_system() {}

#[hot]
fn system_with_commands(commands: Commands) {}

#[hot]
fn system_with_zst_query(query: Query<()>) {}

#[hot]
fn system_with_readonly_query(query: Query<&Transform>) {}

#[hot]
fn system_with_mut_query(mut query: Query<&mut Transform>) {}

#[hot]
fn system_with_mixed_query(query: Query<&Transform>, mut mut_query: Query<&mut Node>) {}

#[hot]
fn system_with_single_query(query: Single<Entity, With<Transform>>) {}

#[hot]
fn system_with_resource(resource: Res<Time>) {}

#[hot]
fn system_with_resource_and_query(resource: Res<Time>, query: Query<&Transform>) {}

#[hot]
fn system_with_mut_resource(mut resource: ResMut<Time>) {}

#[hot]
fn system_with_mut_resource_and_query(mut resource: ResMut<Time>, query: Query<&Transform>) {}

#[hot]
fn system_with_mut_resource_and_mut_query(
    mut resource: ResMut<Time>,
    mut query: Query<&mut Transform>,
) {
}

#[hot]
fn system_with_mut_resource_and_single_query(
    mut resource: ResMut<Time>,
    query: Single<Entity, With<Transform>>,
) {
}

#[hot]
fn system_with_mut_resource_and_mut_single_query(
    mut resource: ResMut<Time>,
    mut query: Single<&mut Transform, With<Transform>>,
) {
}
