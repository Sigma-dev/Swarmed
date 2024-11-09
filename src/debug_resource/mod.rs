use bevy::prelude::*;

struct DebugMessage {
    message: String,
    entity: Entity
}

#[derive(Resource, Default)]
pub struct DebugResource {
    to_debug_queue: Vec<DebugMessage>
}

impl DebugResource {
    pub fn debug(&mut self, entity: Entity, message: impl Into<String>) {
        self.to_debug_queue.push(DebugMessage { entity, message: message.into() });        
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<DebugResource>();
    app.add_systems(Update,debug);
}

fn debug(
    mut debug_res: ResMut<DebugResource>,
    names_query: Query<&Name>
) {
    for to_debug in &debug_res.to_debug_queue {
        let name = names_query.get(to_debug.entity).map_or("UNDEFINED", |n| n.as_str());
        println!("entity: {} name: {}, msg: {}", to_debug.entity, name, to_debug.message);
    }
    debug_res.to_debug_queue.clear();
}