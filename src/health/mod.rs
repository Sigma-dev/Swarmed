use bevy::prelude::*;

mod networked;

#[derive(Component)]
pub struct Health {
    amount: i32,
    max_amount: u32,
    dead: bool,
    queued_health_changes: Vec<QueuedHealthChange>
}

#[derive(Event)]
pub struct HealthChange {
    pub entity: Entity,
    pub change: i32,
    pub new_health: i32,
}
pub struct QueuedHealthChange {
    pub change: i32,
    pub new_health: i32,
}

#[derive(Event)]
pub struct Death {
    pub entity: Entity
}

impl Health {
    pub fn new(max: u32) -> Health {
        Health { amount: max as i32, max_amount: max, dead: false, queued_health_changes: Vec::new() }
    }
    
    pub fn take_damage(&mut self, damage: u32) {
        self.amount -= damage as i32;
        self.queued_health_changes.push(QueuedHealthChange { change: -(damage as i32), new_health: self.amount });
        if self.amount <= 0 {
            self.die();
        }
    }
    
    pub fn heal(&mut self, heal: u32) {
        self.amount = (self.amount + heal as i32).min(self.max_amount as i32);
        self.queued_health_changes.push(QueuedHealthChange { change: heal as i32, new_health: self.amount });
    }

    pub fn die(&mut self) {
        self.dead = true;
    }

    pub fn is_dead(&self) -> bool {
        self.dead
    }

    pub fn get_hp(&self) -> i32 {
        self.amount
    }

    pub fn get_max_hp(&self) -> u32 {
        self.max_amount
    }
}

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, (handle_health_changes, handle_deaths))
        .add_event::<HealthChange>()
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

fn handle_health_changes(
    mut health_query: Query<(Entity, &mut Health)>,
    mut changes_writer: EventWriter<HealthChange>
) {
    for (entity, mut health) in health_query.iter_mut() {
        for queued in &health.queued_health_changes {
            changes_writer.send(HealthChange { entity, change: queued.change, new_health: queued.new_health });
        }
    }
}