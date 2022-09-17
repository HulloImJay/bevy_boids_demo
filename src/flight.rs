use lerp::Lerp;
use bevy::{
    prelude::*,
};

use crate::jay_math;
use crate::velocitator::Velocitator;

/// The Flight plugin.
pub struct Flight;

impl Plugin for Flight {
    fn build(&self, app: &mut App) {
        app
            .add_system(flyer_goals_reduce_to_components_system)
            .add_system(flyer_steering_system.after(flyer_goals_reduce_to_components_system))
            .add_system(flyer_movement_system.after(flyer_steering_system))
            .add_system(flyer_copy_velocity_system.after(flyer_movement_system))
            .register_type::<Flyer>();
    }
}

/// A thing that is flying.
#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Flyer
{
    pub goal_velocity: Vec3,
    pub goal_components: FlyerGoalComponents,
    pub speed_linear: f32,
    pub accel_linear: f32,
    pub ang_x: f32,
    pub ang_y: f32,
    pub ang_x_vel: f32,
    pub ang_y_vel: f32,
    pub props: FlyerProps,
}

/// Flying properties. You can change these during play to simulate different
/// states such as applying new forces or gliding.
#[derive(Reflect, Default)]
pub struct FlyerProps
{
    pub accel_max: f32,
    pub spd_min: f32,
    pub spd_max: f32,
    pub ang_spd_x_max: f32,
    pub ang_spd_y_max: f32,
    pub ang_z_from_y_spd: f32,
}

/// The goal that a flyer would like to achieve, reduced to components.
#[derive(Reflect, Default)]
pub struct FlyerGoalComponents
{
    pub speed_linear: f32,
    pub ang_x: f32,
    pub ang_y: f32,
}

fn flyer_copy_velocity_system(
    mut query: Query<(&Flyer, &Transform, &mut Velocitator)>,
) {
    for (flyer, transform, mut velocitator) in query.iter_mut() {
        velocitator.velocity = flyer.speed_linear * transform.forward();
    }
}

fn flyer_goals_reduce_to_components_system(
    mut query: Query<(&Transform, &mut Flyer)>,
) {
    for (transform, mut flyer) in query.iter_mut() {
        let goal_speed = flyer.goal_velocity.length();
        let goal_direction = if goal_speed > 0.0 { flyer.goal_velocity / goal_speed } else { Vec3::ZERO };
        let vel_dot = goal_direction.dot(transform.forward()).clamp(0.0, 1.0);

        flyer.goal_components.speed_linear = flyer.props.spd_min.lerp(goal_speed, vel_dot).min(flyer.props.spd_max);

        // yaw, pitch
        (flyer.goal_components.ang_y, flyer.goal_components.ang_x) = jay_math::vec3_to_yaw_pitch(goal_direction);
    }
}

fn flyer_steering_system(
    time: Res<Time>,
    mut query: Query<&mut Flyer>,
) {
    for mut flyer in query.iter_mut() {
        let (spd_new, accel_new) = jay_math::smooth_damp(
            flyer.speed_linear,
            flyer.goal_components.speed_linear,
            flyer.accel_linear,
            0.2,
            flyer.props.accel_max,
            time.delta_seconds(),
        );
        flyer.speed_linear = spd_new;
        flyer.accel_linear = accel_new;

        let (ang_x_new, ang_x_vel_new) = jay_math::smooth_damp_angle(
            flyer.ang_x,
            flyer.goal_components.ang_x,
            flyer.ang_x_vel,
            0.2,
            flyer.props.ang_spd_x_max,
            time.delta_seconds(),
        );
        flyer.ang_x = ang_x_new;
        flyer.ang_x_vel = ang_x_vel_new;

        let (ang_y_new, ang_y_vel_new) = jay_math::smooth_damp_angle(
            flyer.ang_y,
            flyer.goal_components.ang_y,
            flyer.ang_y_vel,
            0.2,
            flyer.props.ang_spd_y_max,
            time.delta_seconds(),
        );
        flyer.ang_y = ang_y_new;
        flyer.ang_y_vel = ang_y_vel_new;
    }
}

pub fn flyer_movement_system(
    time: Res<Time>,
    mut query: Query<(&Flyer, &mut Transform)>,
) {
    for (flyer, mut transform) in query.iter_mut() {
        transform.rotation = Quat::from_euler(EulerRot::YXZ, flyer.ang_y, flyer.ang_x, flyer.props.ang_z_from_y_spd * flyer.ang_y_vel);
        transform.translation = transform.translation + transform.forward() * flyer.speed_linear * time.delta_seconds();
    }
}