use bevy::prelude::*;

#[derive(Component)]
pub struct Movable {
    speed_mult: f32
}

impl Default for Movable {
    fn default() -> Self {
        Movable { speed_mult: 1. }
    }
}

impl Movable {
    pub fn new(speed_mult: f32) -> Self {
        Movable { speed_mult }
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update,movable);
}

fn movable(
    mut transform_query: Query<(&mut Transform, &Movable)>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for (mut movable_transform, movable) in transform_query.iter_mut() {
        let mut vec = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) {
            vec.z += 1.0
        }
        if keys.pressed(KeyCode::KeyS) {
            vec.z -= 1.0
        }
        if keys.pressed(KeyCode::KeyD) {
            vec.x -= 1.0
        }
        if keys.pressed(KeyCode::KeyA) {
            vec.x += 1.0
        }
        if keys.pressed(KeyCode::KeyQ) {
            vec.y += 1.0
        }
        if keys.pressed(KeyCode::KeyE) {
            vec.y -= 1.0
        }
        movable_transform.translation += vec * 0.01 * movable.speed_mult;
    }
}