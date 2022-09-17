use bevy::{
    prelude::*,
};

/// A component with a velocity. This is intended to generalise objects with velocity
/// for use in the boids module (so it isn't strictly tied to the hacky flight model
/// and can be reused). It really ought to be a trait, but core bevy does not support
/// queries on traits just yet, so it's a component you have to copy your velocity to 🤷
#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Velocitator {
    pub velocity: Vec3,
}