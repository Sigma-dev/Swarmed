use avian3d::parry::na::coordinates::XYZ;
use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy::window::{CursorGrabMode, PrimaryWindow};

pub struct FpsCameraPlugin;

#[derive(Component)]
pub struct FpsCamera {
    pub sensitivity: f32,
}

impl Plugin for FpsCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_fps_cameras)
        .add_systems(Startup, setup);
    }
}

fn setup(mut q_windows: Query<&mut Window, With<PrimaryWindow>>,) {
    let mut primary_window = q_windows.single_mut();

    primary_window.cursor.grab_mode = CursorGrabMode::Locked;
    primary_window.cursor.visible = false;
}

fn handle_fps_cameras(
    mut query: Query<(Entity, &FpsCamera, Option<&Parent>)>,
    mut motion_evr: EventReader<MouseMotion>,
    mut transform_query: Query<&mut Transform>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    for (entity, free_cam, maybe_parent) in &mut query {
       // let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        for ev in motion_evr.read() {
            let rotation_dir = -ev.delta * free_cam.sensitivity * time.delta_seconds();
           // yaw += rotation_dir.x;
           // pitch += rotation_dir.y;
            if let Some(parent) = maybe_parent {
                let Ok([mut transform, mut parent_transform]) = transform_query.get_many_mut([entity, parent.get()]) else {continue;};
                transform.rotate_axis(Dir3::X, rotation_dir.y);
                parent_transform.rotate_axis(Dir3::Y, rotation_dir.x);
                gizmos.line(transform.translation, transform.translation + parent_transform.forward().as_vec3() * 5., Color::srgb(1., 0., 0.));
                gizmos.line(transform.translation, transform.translation + parent_transform.right().as_vec3() * 5., Color::srgb(0., 1., 0.));
            } else {
                let transform = transform_query.get_mut(entity).unwrap();
                //transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
            }
            
        }
    }
}

fn handle_parent_propagation(
    query: Query<(Entity, &FpsCamera, Option<&Parent>)>,
    mut transform_query: Query<&mut Transform>
) {
    for (entity, fps_camera, maybe_parent) in query.iter() {
        if let Some(parent) = maybe_parent {
           // let [transform, parent_transform] = transform_query.many_mut([entity, parent]);
           // let mut euler = transform.rotation.to_euler(EulerRot::XYZ);
          //  euler.z = 0;
           // parent_transform.rotation = Quat::from_euler(EulerRot::XYZ, euler.x, euler.y, euler.z);
        }
    }
}