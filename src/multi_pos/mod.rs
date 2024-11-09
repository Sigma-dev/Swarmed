use bevy::prelude::*;

#[derive(Clone)]
pub struct MultiPosLerp {
    lerp_speed: f32
}

#[derive(Component, Clone)]
pub struct MultiPosCamera {
    positions: Vec<(Vec3, Vec3)>,
    index: usize,
    lerp: Option<MultiPosLerp>
}

impl MultiPosCamera {
    pub fn new(positions: Vec<(Vec3, Vec3)>) -> Self {
        MultiPosCamera { positions, index: 0, lerp: None }
    }

    pub fn with_lerp(&mut self, lerp_speed: f32) -> Self {
        self.lerp = Some(MultiPosLerp { lerp_speed });
        self.clone()
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update,(on_add, multi_pos));
}

fn on_add(
    mut camera_query: Query<(&mut Transform, &MultiPosCamera), Added<MultiPosCamera>>,
) {
    for (mut transform, multi_pos) in camera_query.iter_mut() {
        if let Some((pos, look_at)) = multi_pos.positions.first() {
            *transform = Transform::from_translation(*pos).looking_at(*look_at, Vec3::Y);
        }
    }
}

fn multi_pos(
    mut camera_query: Query<(&mut Transform, &mut MultiPosCamera)>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for (mut transform, mut multi_pos) in camera_query.iter_mut() {
        if keys.just_pressed(KeyCode::ArrowLeft) {
            if multi_pos.index == 0 {
                multi_pos.index = multi_pos.positions.len();
            }
            multi_pos.index -= 1;
        }
        else if keys.just_pressed(KeyCode::ArrowRight) {
            multi_pos.index += 1;
        }
        multi_pos.index = multi_pos.index.rem_euclid(multi_pos.positions.len());
        let (position, look_at) = multi_pos.positions[multi_pos.index as usize];
        let target = Transform::from_translation(position).looking_at(look_at, Vec3::Y);
        if let Some(lerp) = &multi_pos.lerp {
            transform.translation = transform.translation.lerp(target.translation, lerp.lerp_speed);
            *transform.rotation = *transform.rotation.lerp(target.rotation, lerp.lerp_speed);
        } else {
            *transform = target;
        }
    }
}