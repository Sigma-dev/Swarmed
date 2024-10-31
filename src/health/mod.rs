use bevy::prelude::*;

#[derive(Component)]
pub struct Health {
    amount: i32,
    max_amount: u32,
    dead: bool
}

#[derive(Event)]
pub struct Death {
    pub entity: Entity
}

impl Health {
    pub fn new(max: u32) -> Health {
        Health { amount: max as i32, max_amount: max, dead: false }
    }
    
    pub fn take_damage(&mut self, damage: u32) {
        self.amount -= damage as i32;
        if self.amount <= 0 {
            self.die();
        }
    }
    
    pub fn heal(&mut self, heal: u32) {
        self.amount =( self.amount + heal as i32).min(self.max_amount as i32);
    }

    pub fn die(&mut self) {
        self.dead = true;
    }

    pub fn is_dead(&self) -> bool {
        self.dead
    }
}

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, handle_deaths)
        .add_event::<Death>();
    }
}

fn handle_deaths(
    mut commands: Commands,
    health_query: Query<(Entity, &Health), Changed<Health>>,
    mut deaths_writer: EventWriter<Death>
) {
    for (entity, health) in health_query.iter() {
        if health.is_dead() {
            commands.get_entity(entity).unwrap().despawn();
            deaths_writer.send(Death { entity: entity });
        }
    }
}