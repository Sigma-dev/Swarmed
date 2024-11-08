use bevy::prelude::*;
use networked::NetworkedHealthPlugin;

mod networked;

#[derive(PartialEq)]
enum DeathState {
    Alive,
    Dying,
    Dead
}

#[derive(Component)]
pub struct Health {
    amount: i32,
    max_amount: u32,
    dead: DeathState,
    destroy_on_death: bool,
    queued_health_changes: Vec<QueuedHealthChange>
}

#[derive(Event)]
pub struct HealthChange {
    pub entity: Entity,
    pub change: i32,
    pub new_health: i32,
    pub authentic: bool,
}
pub struct QueuedHealthChange {
    pub change: i32,
    pub new_health: i32,
    pub authentic: bool,
}

#[derive(Event)]
pub struct Death {
    pub entity: Entity
}

impl Health {
    pub fn new(max: u32, destroy_on_death: bool) -> Health {
        Health { amount: max as i32, max_amount: max, dead: DeathState::Alive, queued_health_changes: Vec::new(), destroy_on_death }
    }
    
    pub fn take_damage(&mut self, damage: u32, authentic: bool) {
        self.amount -= damage as i32;
        if authentic {
            self.queued_health_changes.push(QueuedHealthChange { change: -(damage as i32), new_health: self.amount, authentic });   
        }
        if self.amount <= 0 {
            self.die();
        }
    }
    
    pub fn heal(&mut self, heal: u32, authentic: bool) {
        self.amount = (self.amount + heal as i32).min(self.max_amount as i32);
        if authentic {
            self.queued_health_changes.push(QueuedHealthChange { change: heal as i32, new_health: self.amount, authentic });
        }
    }

    pub fn change(&mut self, change: i32, authentic: bool) {
        if change > 0 {
            self.heal(change as u32, authentic)
        } else {
            self.take_damage(-change as u32, authentic);
        }
    }

    pub fn set_health(&mut self, new: i32, authentic: bool) -> Result<(), ()> {
        if new > self.max_amount as i32 {
            return Err(());
        }
        if new < 0 {
            return Err(());
        }
        let diff = new - self.get_hp();
        self.change(diff, authentic);
        return Ok(());
    }
    
    pub fn reset(&mut self, authentic: bool) {
        self.set_full_hp(authentic);
        self.dead = DeathState::Alive;
    }

    pub fn set_full_hp(&mut self, authentic: bool) {
        let _ = self.set_health(self.max_amount as i32, authentic);
    }

    pub fn die(&mut self) {
        if self.dead == DeathState::Alive {
            self.dead = DeathState::Dying
        }
    }

    pub fn is_dead(&self) -> bool {
        !(self.dead == DeathState::Alive)
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
        .add_plugins(NetworkedHealthPlugin)
        .add_systems(Update, (handle_health_changes, handle_deaths))
        .add_event::<HealthChange>()
        .add_event::<Death>();
    }
}

fn handle_deaths(
    mut commands: Commands,
    mut health_query: Query<(Entity, &mut Health), Changed<Health>>,
    mut deaths_writer: EventWriter<Death>
) {
    for (entity, mut health) in health_query.iter_mut() {
        if health.dead == DeathState::Dying {
            if health.destroy_on_death {
                commands.get_entity(entity).unwrap().despawn();
            }
            deaths_writer.send(Death { entity: entity });
            health.dead = DeathState::Dead;
        }
    }
}

fn handle_health_changes(
    mut health_query: Query<(Entity, &mut Health)>,
    mut changes_writer: EventWriter<HealthChange>
) {
    for (entity, health) in health_query.iter_mut() {
        for queued in &health.queued_health_changes {
            changes_writer.send(HealthChange { entity, change: queued.change, new_health: queued.new_health, authentic: true });
        }
    }
}