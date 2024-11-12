use bevy::prelude::*;
use bevy_mod_picking::prelude::PointerInteraction;

use super::LegCreature;

#[derive(Component)]
pub struct TargetControl;

#[derive(Resource, Default)]
pub struct TargetPosition {
    target: Option<Vec3>
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update,(handle_target, follow_target));
    app.init_resource::<TargetPosition>();
}

pub(crate) fn follow_target(
    mut creature_query: Query<(&mut Transform, &mut LegCreature), With<TargetControl>>,
    mut target_resource: ResMut<TargetPosition>,
    time: Res<Time>,
) {
    for (mut transform, mut creature) in creature_query.iter_mut() {
        creature.target_offset = Vec3::ZERO;
        let Some(target_position) = target_resource.target else {continue;};
        if target_position.distance(transform.translation) < 0.1 {
            target_resource.target = None;
            return;
        }
        let vec = (target_position - transform.translation).normalize();
        let rotation = transform.forward().xz().angle_between(vec.xz()) * 0.1;
        creature.target_offset = (-Vec3::Z * creature.speed_mult * 1.).clamp_length(0., 1.);
        transform.translation += vec * creature.speed_mult * 10. * time.delta_seconds();
        transform.rotate_local_y(-rotation * 40. * time.delta_seconds());
    }
}

pub(crate) fn handle_target(
    mut target_resource: ResMut<TargetPosition>,
    pointers: Query<&PointerInteraction>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if !buttons.just_pressed(MouseButton::Left) { return };
    for point in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
        .filter_map(|(_, hit)| hit.position)
    {
        target_resource.target = Some(point);
    }
}