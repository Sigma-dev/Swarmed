use bevy::prelude::*;
use weapon_inventory::WeaponInventory;

pub mod weapon;
pub mod weapon_inventory;

pub struct WeaponSystemPlugin;

impl Plugin for WeaponSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_inputs);
    }
}

#[derive(Component)]
pub struct WeaponSystem {
    pub(crate) inventory: WeaponInventory
}

fn handle_inputs(
    mut weapon_systems_query: Query<&mut WeaponSystem>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>
) {
    for mut weapon_system in weapon_systems_query.iter_mut() {
        if mouse.just_pressed(MouseButton::Left) {
            weapon_system.inventory.try_fire(time.elapsed_seconds()).map_err(|e| println!("{:?}", e));
        }
        if keys.just_pressed(KeyCode::KeyR) {
            weapon_system.inventory.try_reload(time.elapsed_seconds()).map_err(|e| println!("{:?}", e));
        }
        if keys.just_pressed(KeyCode::Digit1) {
            weapon_system.inventory.swap_weapon(time.elapsed_seconds(), 0).map_err(|e| println!("{:?}", e));
        }
    }
}