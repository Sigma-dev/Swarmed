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
    mut query: Query<(&mut Transform, &FpsCamera)>,
    mut motion_evr: EventReader<MouseMotion>,
    time: Res<Time>
) {
    for (mut transform, free_cam) in &mut query {
        let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        for ev in motion_evr.read() {
            let rotation_dir = -ev.delta * free_cam.sensitivity * time.delta_seconds();
            yaw += rotation_dir.x;
            pitch += rotation_dir.y;
            transform.rotation =
            Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
        }
    }
}