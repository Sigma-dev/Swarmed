use super::weapon::{self, ReloadError, Weapon};
pub struct WeaponInventory {
    weapons: Vec<Weapon>,
    equipped_weapon_index: Option<usize>,
    last_swap_start: Option<f32>,
    last_swap_duration: Option<f32>,
}

impl WeaponInventory {
    pub fn new(weapons: Vec<Weapon>) -> WeaponInventory {
        WeaponInventory {
            weapons,
            equipped_weapon_index: None,
            last_swap_start: None,
            last_swap_duration: None
        }
    }

    pub fn add_weapon(&mut self, weapon: Weapon) {
        self.weapons.push(weapon);
    }

    pub fn swap_weapon(&mut self, time: f32, index: usize) -> Result<(), SwapError> {
        if index > self.weapons.len() { return Err(SwapError::WeaponUnavailable) }
        if let Some(current_index) = self.equipped_weapon_index {
            if index == current_index { return Err(SwapError::SameWeapon) }
        };
        if self.is_swapping(time) { return Err(SwapError::AlreadySwapping) }
        let unequip_time = self.get_equipped_weapon().map(|weapon| weapon.get_unequip_time()).unwrap_or(0.);
        let Some(equip_time) = self.get_weapon(index).map(|weapon| weapon.get_equip_time()) else { return Err(SwapError::WeaponUnavailable) };
        self.last_swap_duration = Some(unequip_time + equip_time);
        self.last_swap_start = Some(time);
        self.equipped_weapon_index = Some(index);
        Ok(())
    }

    pub fn try_fire(&mut self, time: f32) -> Result<(), weapon::FireError> {
        if self.is_swapping(time) { return Err(weapon::FireError::SwappingWeapons)};
        let Some(weapon) = self.get_equipped_weapon() else { return Err(weapon::FireError::WeaponUnavailable)};
        weapon.try_fire(time)
    }

    pub fn try_reload(&mut self, time: f32) -> Result<(), weapon::ReloadError> {
        if self.is_swapping(time) { return Err(weapon::ReloadError::SwappingWeapons)};
        let Some(weapon) = self.get_equipped_weapon() else { return Err(weapon::ReloadError::WeaponUnavailable)};
        weapon.try_reload(time)
    }

    pub fn is_swapping(&self, time: f32) -> bool {
        let Some(last_swap_start) = self.last_swap_start else { return false; };
        let Some(last_swap_duration) = self.last_swap_duration else { return false; };
        time < last_swap_start + last_swap_duration
    }

    fn get_equipped_weapon(&mut self) -> Option<&mut Weapon> {
        self.weapons.get_mut(self.equipped_weapon_index?)
    }

    fn get_weapon(&mut self, index: usize) -> Option<&mut Weapon> {
        self.weapons.get_mut(index)
    }
}

#[derive(Debug)]
pub enum SwapError {
    SameWeapon,
    AlreadySwapping,
    WeaponUnavailable,
}