// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

/// A crude boids demo in Bevy.
/// Created for Dinacon 2022 in Sri Lanka.

mod anim;
mod boids;
mod observe;
mod bounds;
mod flight;
mod jay_math;
mod velocitator;
mod crows;

use std::f32::consts::{PI, TAU};

use anim::*;
use boids::*;
use observe::*;
use bounds::*;
use flight::*;
use crows::*;
use bevy::{
    prelude::*,
    render::camera::Viewport,
    window::{WindowId, WindowResized},
    core_pipeline::clear_color::ClearColorConfig,
    render::camera::ScalingMode,
};
use big_brain::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use rand::prelude::*;

pub const LAUNCHER_TITLE: &str = "Bevy Boids Demo";

#[derive(Component)]
struct TopDownCam;

#[derive(Component)]
struct SideCam;

#[derive(Component)]
struct WidePerspectiveCamera;

#[derive(Component)]
struct FollowCamera
{
    target_entity: Entity,
}

fn main() {

    // The overall bounds of our simulation.
    let dem_bounds = Bounds::new(
        50.0,
        0.0,
        600.0,
        0.0,
        250.0,
        0.0,
        600.0,
        50.0,
    );
    
    App::new()
        .insert_resource(WindowDescriptor {
            title: LAUNCHER_TITLE.to_string(),
            canvas: Some("#bevy".to_string()),
            fit_canvas_to_parent: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(load_icon)
        .add_plugin(BigBrainPlugin)
        .add_plugin(JayAnimation)
        .add_plugin(Observe)
        .add_plugin(Boids)
        .add_plugin(Flight)
        .add_plugin(EguiPlugin)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0,
        })
        .insert_resource(StuffsToObserve::new(dem_bounds.cells_x, dem_bounds.cells_z, dem_bounds.cell_size))
        .insert_resource(ClearColor(Color::rgb(1.0, 0.8, 0.5)))
        .insert_resource(dem_bounds)
        .insert_resource(CrowGlobalProps {
            separation_weight: 2.0,
            alignment_weight: 2.0,
            cohesion_weight: 2.0,
            keep_in_bounds_weight: 0.5,
            keep_level_weight: 0.5,
        })
        .add_startup_system(startup)
        .add_system(crows::flyer_goal_velocity_from_boids_system)
        .add_system(keep_in_bounds_system)
        .add_system(keep_level_system)
        .add_system(stamina_update_system)
        .add_system(set_camera_viewports_system)
        .add_system(crow_ui_system)
        .add_system(stamina_update_system)
        .add_system(follow_cam_system.after(flyer_movement_system))
        .add_system_to_stage(BigBrainStage::Actions, flap_action_system)
        .add_system_to_stage(BigBrainStage::Scorers, flap_scorer_system)
        .run();
}


fn load_icon(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("icon.png"),
        ..default()
    });
}

fn crow_ui_system(
    mut egui_context: ResMut<EguiContext>,
    mut common_props: ResMut<CrowGlobalProps>,
) {
    egui::Window::new("House Crows").show(egui_context.ctx_mut(), |ui| {
        ui.label("Boids Weights:");
        ui.add(egui::Slider::new(&mut common_props.separation_weight, 0.0..=5.0).text("separation"));
        ui.add(egui::Slider::new(&mut common_props.alignment_weight, 0.0..=5.0).text("alignment"));
        ui.add(egui::Slider::new(&mut common_props.cohesion_weight, 0.0..=5.0).text("cohesion"));
        ui.label("Other Weights:");
        ui.add(egui::Slider::new(&mut common_props.keep_in_bounds_weight, 0.0..=2.0).text("keep in bounds"));
        ui.add(egui::Slider::new(&mut common_props.keep_level_weight, 0.0..=2.0).text("keep level"));
    });
}


fn follow_cam_system(
    mut query_cameras: Query<(&Camera, &mut Transform, &mut FollowCamera)>,
    query_target: Query<(&Flyer, &Transform, Entity), Without<FollowCamera>>,
)
{
    for (_, mut camera_transform, mut follow_camera) in query_cameras.iter_mut() {
        if let Ok((_, flyer_transform, _)) = query_target.get(follow_camera.target_entity) {
            camera_transform.translation = flyer_transform.translation + flyer_transform.forward() * 4.0 - flyer_transform.right() * 4.0 + flyer_transform.up() * 2.0;
            camera_transform.look_at(flyer_transform.translation, -Vec3::Y);
            return;
        }
        for (_, _, entity) in query_target.iter()
        {
            follow_camera.target_entity = entity;
            return;
        }
    }
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    bounds: Res<Bounds>,
) {
    let mid_point = Vec3::new(bounds.x_min + 0.5 * bounds.x_size, bounds.y_min + 0.5 * bounds.y_size, bounds.z_min + 0.5 * bounds.z_size);
    let mid_top = Vec3::new(mid_point.x, bounds.y_max, mid_point.z);

    // Camera 1 — top view
    commands.spawn_bundle(Camera3dBundle {
        projection: OrthographicProjection {
            scale: bounds.z_size * 1.20,
            scaling_mode: ScalingMode::FixedVertical(1.0),
            ..default()
        }
            .into(),
        camera: Camera {
            priority: 1, // rendering order
            ..default()
        },
        transform: Transform::from_translation(mid_top)
            .with_rotation(Quat::from_axis_angle(Vec3::X, -PI * 0.5)),

        ..default()
    })
        .insert(TopDownCam);


    let side_cam_pos = Vec3::new(mid_point.x, mid_point.y, bounds.z_max);

    // Camera 2 — side view
    commands.spawn_bundle(Camera3dBundle {
        projection: OrthographicProjection {
            scale: bounds.x_size * 1.20,
            scaling_mode: ScalingMode::FixedHorizontal(1.0),
            ..default()
        }
            .into(),
        camera: Camera {
            priority: 2, // rendering order
            ..default()
        },
        camera_3d: Camera3d {
            // dont clear on the second camera because the first camera already cleared the window
            clear_color: ClearColorConfig::None,
            ..default()
        },
        transform: Transform::from_translation(side_cam_pos),
        ..default()
    })
        .insert(SideCam);

    // Perspective view.
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0.0, bounds.y_min + 0.5 * bounds.y_size, 0.0)
            .looking_at(mid_point, Vec3::Y),
        camera: Camera {
            priority: 3, // rendering order
            ..default()
        },
        camera_3d: Camera3d {
            // dont clear on the second camera because the first camera already cleared the window
            clear_color: ClearColorConfig::None,
            ..default()
        },
        ..default()
    })
        .insert(WidePerspectiveCamera);

    // Follow view.
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0.0, bounds.y_min + 0.5 * bounds.y_size, 0.0)
            .looking_at(mid_point, Vec3::Y),
        camera: Camera {
            priority: 4, // rendering order
            ..default()
        },
        camera_3d: Camera3d {
            // dont clear on the second camera because the first camera already cleared the window
            clear_color: ClearColorConfig::None,
            ..default()
        },
        projection: PerspectiveProjection {
            aspect_ratio: 1.0,
            near: 0.3,
            far: 1000.0,
            fov: 80.0,
        }.into(),
        ..default()
    })
        .insert(FollowCamera
        {
            target_entity: Entity::from_raw(1000)
        });

    // Box ground.
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box {
            min_x: bounds.x_min,
            max_x: bounds.x_max,
            min_z: bounds.z_min,
            max_z: bounds.z_max,
            min_y: -10.0,
            max_y: 0.0,
        })),
        material: materials.add(Color::rgb(0.9, 0.7, 0.4).into()),
        transform: Transform::identity(),
        ..default()
    });

    // Light
    commands.spawn_bundle(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            1.0,
            -std::f32::consts::FRAC_PI_4,
        )),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 33000.0,
            ..default()
        },
        ..default()
    });


    let count = 180;
    let mut rng = rand::thread_rng();

    for _ in 0..count
    {
        let m = bounds.margin + 50.0;
        let x = rng.gen_range(bounds.x_min + m..bounds.x_max - m);
        let y = rng.gen_range(bounds.y_min + m..bounds.y_max - m);
        let z = rng.gen_range(bounds.z_min + m..bounds.z_max - m);

        let rot = rng.gen_range(-TAU..TAU) * 0.5;

        let pos = Vec3::from((x, y, z));

        make_instance(
            &mut commands,
            &asset_server,
            "house_crow.glb",
            pos,
            Quat::from_axis_angle(Vec3::Y, rot),
        );
    }
}

fn set_camera_viewports_system(
    windows: Res<Windows>,
    mut resize_events: EventReader<WindowResized>,
    mut top_down_cam: Query<&mut Camera, (With<TopDownCam>, Without<FollowCamera>, Without<SideCam>, Without<WidePerspectiveCamera>)>,
    mut wide_perspective_cam: Query<&mut Camera, (With<WidePerspectiveCamera>, Without<FollowCamera>, Without<SideCam>, Without<TopDownCam>)>,
    mut side_cam: Query<&mut Camera, (With<SideCam>, Without<FollowCamera>, Without<WidePerspectiveCamera>, Without<TopDownCam>)>,
    mut follow_cam: Query<&mut Camera, (With<FollowCamera>, Without<SideCam>, Without<WidePerspectiveCamera>, Without<TopDownCam>)>,
) {
    // We need to dynamically resize the camera's viewports whenever the window size changes
    // so then each camera always takes up half the screen.
    // A resize_event is sent when the window is first created, allowing us to reuse this system for initial setup.
    for resize_event in resize_events.iter() {
        if resize_event.id == WindowId::primary() {
            let window = windows.primary();
            let mut top_down_cam = top_down_cam.single_mut();
            top_down_cam.viewport = Some(Viewport {
                physical_position: UVec2::new(0, window.physical_height() / 3),
                physical_size: UVec2::new(window.physical_width() / 2, window.physical_height() / 3 * 2),
                ..default()
            });

            let mut side_cam = side_cam.single_mut();
            side_cam.viewport = Some(Viewport {
                physical_position: UVec2::new(0, 0),
                physical_size: UVec2::new(window.physical_width() / 2, window.physical_height() / 3),
                ..default()
            });

            let mut right_camera = wide_perspective_cam.single_mut();
            right_camera.viewport = Some(Viewport {
                physical_position: UVec2::new(window.physical_width() / 2, window.physical_height() / 2),
                physical_size: UVec2::new(window.physical_width() / 2, window.physical_height() / 2),
                ..default()
            });

            let mut follow_cam = follow_cam.single_mut();
            follow_cam.viewport = Some(Viewport {
                physical_position: UVec2::new(window.physical_width() / 2, 0),
                physical_size: UVec2::new(window.physical_width() / 2, window.physical_height() / 2),
                ..default()
            });
        }
    }
}
