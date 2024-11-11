use bevy::prelude::*;

use super::LegCreature;

#[derive(Component)]
pub struct ManualControl;

pub fn plugin(app: &mut App) {
    app.add_systems(Update,move_creature);
}

pub(crate) fn move_creature(
    mut creature_query: Query<(&mut Transform, &mut LegCreature), With<ManualControl>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for (mut transform, mut creature) in creature_query.iter_mut() {
        let mut vec = Vec3::ZERO;
        let mut rotation = 0.;
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
            rotation = 1.
        }
        if keys.pressed(KeyCode::KeyE) {
            rotation = -1.
        }
        if vec != Vec3::ZERO { vec = vec.normalize(); };
        creature.target_offset = (-vec * creature.speed_mult * 1.).clamp_length(0., 1.);
        let copy = transform.clone();
        transform.translation += ((copy.forward() * vec.z) + (copy.right() * vec.x))  * creature.speed_mult * 0.05;
        transform.rotate_local_y(rotation * 0.01);
    }
}