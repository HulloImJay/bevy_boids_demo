use bevy::{
    prelude::*,
};
use big_brain::prelude::*;
use crate::{anim, jay_math};
use anim::*;
use crate::boids;
use boids::*;
use crate::observe;
use observe::*;
use crate::bounds;
use bounds::*;
use crate::flight;
use flight::*;
use crate::velocitator::Velocitator;

/// Some global properties for our crows to use. Makes it possible
/// to have a little UI with sliders to scale the weights even though
/// individuals have their own weights, too.
pub struct CrowGlobalProps
{
    pub separation_weight: f32,
    pub alignment_weight: f32,
    pub cohesion_weight: f32,
    pub keep_in_bounds_weight: f32,
    pub keep_level_weight: f32,
}

// Stamina state.
#[derive(Component, Debug)]
pub struct Stamina {
    recover_per_second: f32,
    current_value: f32,
}

pub fn stamina_update_system(time: Res<Time>, mut staminas: Query<&mut Stamina>) {
    for mut stamina in staminas.iter_mut() {
        stamina.current_value += stamina.recover_per_second * time.delta_seconds();
        if stamina.current_value >= 100.0 {
            stamina.current_value = 100.0;
        }
        // println!("Stamina: {}", stamina.current_value);
    }
}

/// Our flapping action. 
#[derive(Clone, Component, Debug)]
pub struct Flap {
    stamina_usage_per_sec: f32,
}

pub fn flap_action_system(
    time: Res<Time>,
    mut commands: Commands,
    mut staminas: Query<(&mut Stamina, Entity)>,
    mut query_actor_stuff: Query<(&Actor, &mut ActionState, &Flap)>,
) {
    for (Actor(actor), mut state, flap) in query_actor_stuff.iter_mut() {
        if let Ok((mut stamina, entity)) = staminas.get_mut(*actor) {
            match *state {
                ActionState::Requested => {
                    *state = ActionState::Executing;
                    commands.entity(entity).insert(StartAnim {
                        name: String::from("Flap"),
                        loop_plz: true,
                    });
                    commands.entity(entity).insert(ModelSpawned {});
                }
                ActionState::Executing => {
                    stamina.current_value -=
                        flap.stamina_usage_per_sec * time.delta_seconds();
                    if stamina.current_value <= 50.0 {
                        *state = ActionState::Success;
                        commands.entity(entity).insert(StartAnim {
                            name: String::from("Soar"),
                            loop_plz: true,
                        });
                    }
                }
                // All Actions should make sure to handle cancellations!
                ActionState::Cancelled => {
                    *state = ActionState::Failure;
                }
                _ => {}
            }
        }
    }
}

/// The state of wanting to flap
#[derive(Clone, Component, Debug)]
pub struct FlapScorer;

// Looks familiar? It's a lot like Actions!
pub fn flap_scorer_system(
    query_components: Query<(&Stamina, &Flyer)>,
    mut query_actor_stuff: Query<(&Actor, &mut Score), With<FlapScorer>>,
) {
    for (Actor(actor), mut score) in query_actor_stuff.iter_mut() {
        if let Ok((stamina, flyer)) = query_components.get(*actor) {
            let stamina_factor = jay_math::inv_lerp(50.0, 100.0, stamina.current_value).clamp(0.0, 1.0);
            let speed_factor = jay_math::inv_lerp(0.0, 3.0, flyer.goal_velocity.length() - flyer.speed_linear).clamp(0.0, 1.0);

            // The score here must be between 0.0 and 1.0.
            score.set(stamina_factor * speed_factor);
        }
    }
}

/// A simple goal to keep the crows relatively level, in lieu of an actual gravity/lift model.
#[derive(Component, Debug)]
pub struct KeepLevel {
    pub target_vel: Vec3,
    pub weight: f32,
}

/// The system which updates the target velocity for KeepLevel.
pub fn keep_level_system(
    mut query: Query<(&Transform, &Flyer, &mut KeepLevel)>,
)
{
    for (transform, flyer, mut keep_level) in query.iter_mut() {
        let uprightness = Vec3::Y.dot(transform.forward());

        let max = flyer.props.spd_max;

        keep_level.target_vel = -Vec3::Y * uprightness * max;
    }
}

/// A simple goal to keep the crows within some bounds for our demo.
#[derive(Component, Debug)]
pub struct KeepInBounds {
    pub target_vel: Vec3,
    pub weight: f32,
}

/// The system which updates the target and weight on KeepInBounds.
pub fn keep_in_bounds_system(
    mut query: Query<(&Transform, &Flyer, &mut KeepInBounds)>,
    bounds: Res<Bounds>,
)
{
    for (transform, flyer, mut keep_in_bounds) in query.iter_mut() {
        let vel = flyer.speed_linear * transform.forward();

        let max = flyer.props.spd_max;

        keep_in_bounds.target_vel = Vec3::ZERO;
        keep_in_bounds.weight = 0.0;

        if transform.translation.x < bounds.x_min + bounds.margin {
            let t = jay_math::inv_lerp(bounds.x_min + bounds.margin, bounds.x_min, transform.translation.x);
            keep_in_bounds.target_vel.x += max * t;
            keep_in_bounds.weight = 1.0;
        }
        if transform.translation.x > bounds.x_max - bounds.margin {
            let t = jay_math::inv_lerp(bounds.x_max - bounds.margin, bounds.x_max, transform.translation.x);
            keep_in_bounds.target_vel.x -= max * t;
            keep_in_bounds.weight = 1.0;
        }
        if transform.translation.y < bounds.y_min + bounds.margin {
            let t = jay_math::inv_lerp(bounds.y_min + bounds.margin, bounds.y_min, transform.translation.y);
            keep_in_bounds.target_vel.y += max * t;
            keep_in_bounds.weight = 1.0;
        }
        if transform.translation.y > bounds.y_max - bounds.margin {
            let t = jay_math::inv_lerp(bounds.y_max - bounds.margin, bounds.y_max, transform.translation.y);
            keep_in_bounds.target_vel.y -= max * t;
            keep_in_bounds.weight = 1.0;
        }
        if transform.translation.z < bounds.z_min + bounds.margin {
            let t = jay_math::inv_lerp(bounds.z_min + bounds.margin, bounds.z_min, transform.translation.z);
            keep_in_bounds.target_vel.z += max * t;
            keep_in_bounds.weight = 1.0;
        }
        if transform.translation.z > bounds.z_max - bounds.margin {
            let t = jay_math::inv_lerp(bounds.z_max - bounds.margin, bounds.z_max, transform.translation.z);
            keep_in_bounds.target_vel.z -= max * t;
            keep_in_bounds.weight = 1.0;
        }

        keep_in_bounds.target_vel = keep_in_bounds.target_vel - vel;
    }
}

/// Converts various goals (including boids and others) to the goal velocity
/// for a "flyer".
pub fn flyer_goal_velocity_from_boids_system(
    time: Res<Time>,
    common_props: Res<CrowGlobalProps>,
    mut query: Query<(&mut Flyer, &Separation, &Alignment, &Cohesion, &KeepInBounds, &KeepLevel)>,
)
{
    for (mut flyer, separation, alignment, cohesion, keep_in_bounds, keep_level) in query.iter_mut() {

        // Add up the goals.
        flyer.goal_velocity += time.delta().as_secs_f32() *
            (separation.separation_factor * separation.weight * common_props.separation_weight
                + alignment.alignment_factor * alignment.weight * common_props.alignment_weight
                + cohesion.cohesion_factor * cohesion.weight * common_props.cohesion_weight
                + keep_in_bounds.target_vel * keep_in_bounds.weight * common_props.keep_in_bounds_weight
                + keep_level.target_vel * keep_level.weight * common_props.keep_level_weight
            );

        // Clamp our goal velocity within our properties.
        let mag_sqr = flyer.goal_velocity.length_squared();
        if mag_sqr > (flyer.props.spd_max * flyer.props.spd_max)
        {
            let vel_n = flyer.goal_velocity.normalize();
            flyer.goal_velocity = vel_n * flyer.props.spd_max;
        } else if mag_sqr < (flyer.props.spd_min * flyer.props.spd_min)
        {
            let vel_n = flyer.goal_velocity.normalize();
            flyer.goal_velocity = vel_n * flyer.props.spd_min;
        }
    }
}

/// Makes a single crow instance.
pub fn make_instance(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    model_filename: &str,
    position: Vec3,
    rotation: Quat,
) {
    let spd = 75.0;
    let fwd = rotation * -Vec3::Z;
    let (yaw, pitch) = jay_math::vec3_to_yaw_pitch(fwd);

    let gltf = asset_server.load(model_filename);
    commands.spawn_bundle((
        ModelGLTF {
            handle: gltf,
        },
        ModelWaitingToSpawn {},
        StartAnim {
            name: String::from("Soar"),
            loop_plz: true,
        },
        Name::new(format!("House Crow")),
        Observer {
            ..Default::default()
        },
        Separation {
            separation_factor: Vec3::ZERO,
            weight: 0.10,
        },
        Alignment {
            alignment_factor: Vec3::ZERO,
            weight: 0.20,
        },
        Cohesion {
            cohesion_factor: Vec3::ZERO,
            weight: 0.02,
        },
        Flyer {
            speed_linear: spd,
            accel_linear: 0.0,
            ang_x: pitch,
            ang_y: yaw,
            ang_x_vel: 0.0,
            ang_y_vel: 0.0,
            goal_velocity: fwd * spd,
            props: FlyerProps {
                accel_max: 3.0,
                spd_min: 50.0,
                spd_max: 100.0,
                ang_spd_x_max: 0.7,
                ang_spd_y_max: 3.0,
                ang_z_from_y_spd: 0.33,
            },
            ..Default::default()
        },
        Velocitator {
            velocity: fwd * spd,
        },
        KeepInBounds
        {
            target_vel: Vec3::ZERO,
            weight: 0.0,
        },
        KeepLevel
        {
            target_vel: Vec3::ZERO,
            weight: 0.1,
        },
        Stamina
        {
            recover_per_second: 10.0,
            current_value: 70.0,
        },
        Thinker::build()
            .picker(FirstToScore { threshold: 0.6 })
            .when(
                FlapScorer,
                Flap {
                    stamina_usage_per_sec: 30.0,
                },
            ),
    )).insert_bundle(
        SpatialBundle {
            transform: Transform {
                translation: position,
                rotation,
                scale: Vec3::ONE * 0.1,
            },
            ..Default::default()
        });
}
