use bevy::{animation::animate_targets, prelude::*};
use gltf::{handle_weapon_events, handle_weapon_spawn, pre_spawn};
use weapon_inventory::WeaponInventory;

pub mod weapon;
pub mod weapon_inventory;
pub mod gltf;

pub struct WeaponSystemPlugin;

impl Plugin for WeaponSystemPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, (handle_inputs, handle_weapon_events,pre_spawn, handle_weapon_spawn.before(animate_targets)))
        .add_event::<WeaponEvent>();
    }
}

#[derive(Component)]
pub struct WeaponSystem {
    pub(crate) inventory: WeaponInventory
}

fn handle_inputs(
    mut weapon_systems_query: Query<(Entity, &mut WeaponSystem)>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut weapon_events_writer: EventWriter<WeaponEvent>
) {
    for (system_entity, mut weapon_system) in weapon_systems_query.iter_mut() {
        if mouse.just_pressed(MouseButton::Left) {
            if weapon_system.inventory.try_fire(time.elapsed_seconds()).map_err(|e| println!("{:?}", e)).is_ok() {
                weapon_events_writer.send(WeaponEvent {
                    event_type: WeaponEventType::Shoot,
                    system_entity,
                    weapon_index: weapon_system.inventory.get_equipped_index().unwrap(),
                });
            }
        }
        if keys.just_pressed(KeyCode::KeyR) {
            if weapon_system.inventory.try_reload(time.elapsed_seconds()).map_err(|e| println!("{:?}", e)).is_ok() {
                weapon_events_writer.send(WeaponEvent {
                    event_type: WeaponEventType::StartReload,
                    system_entity,
                    weapon_index: weapon_system.inventory.get_equipped_index().unwrap(),
                });
            }
        }
        if keys.just_pressed(KeyCode::Digit1) {
            if weapon_system.inventory.swap_weapon(time.elapsed_seconds(), 0).map_err(|e| println!("{:?}", e)).is_ok() {
                weapon_events_writer.send(WeaponEvent {
                    event_type: WeaponEventType::Equip,
                    system_entity,
                    weapon_index: 0,
                });
            }
        }
    }
}

#[derive(Event)]
pub struct WeaponEvent {
    event_type: WeaponEventType,
    system_entity: Entity,
    weapon_index: usize,
}

pub enum WeaponEventType {
    StartReload,
    Shoot,
    Equip
}