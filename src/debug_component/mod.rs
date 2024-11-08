use bevy::prelude::*;

#[derive(Component)]
pub struct DebugComponent;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, debug);
    }
}

fn debug(
    debug_query: Query<(Entity, Option<&Name>, Option<&GlobalTransform>, Option<&Transform>), With<DebugComponent>>
) {
    for (entity, maybe_name, maybe_gt, maybe_t) in debug_query.iter() {
        let name = maybe_name.map_or("".to_string(), |name| format!(" {}", name.as_str()));
        let gt = maybe_gt.map_or("".to_string(), |gt| format!("GT: {:.2} GR: {:.2} GS: {:.2}", gt.translation(), gt.to_scale_rotation_translation().1, gt.to_scale_rotation_translation().0));
        let t = maybe_t.map_or("".to_string(), |t| format!("T: {:.2} R: {:.2} S: {:.2}", t.translation, t.rotation, t.scale));
        println!("{}{}: {} {}", entity, name, gt, t);
    }
}