use std::cmp::min;

pub struct Weapon {
    pub(crate) characteristics: WeaponCharacteristics,
    pub(crate) ammo_loaded: u32,
    pub(crate) ammo_left: u32,
    pub(crate) visual_identifier: String,
    pub(crate) last_fire_time: Option<f32>,
    pub(crate) last_reload_time: Option<f32>,
}

pub struct WeaponCharacteristics {
    pub(crate) max_loaded: u32,
    pub(crate)max_ammo: u32,
    pub(crate)fire_cd: f32,
    pub(crate)reload_time: f32,
    pub(crate)equip_time: f32,
    pub(crate)unequip_time: f32,
    pub(crate)reloading_empties_mag: bool,
}

impl Weapon {
    pub fn new(characteristics: WeaponCharacteristics, start_ammo: u32, starts_loaded: bool, visual_identifier: impl Into<String>, ) -> Weapon {
        let ammo_loaded = if starts_loaded { characteristics.max_loaded } else { 0 };
        Weapon {
            characteristics,
            ammo_loaded,
            ammo_left: start_ammo,
            visual_identifier: visual_identifier.into(),
            last_fire_time: None,
            last_reload_time: None,
        }
    }

    pub fn try_fire(&mut self, time: f32) -> Result<(), FireError> {
        self.can_fire(time)?;
        self.last_fire_time = Some(time);
        self.ammo_loaded -= 1;
        println!("Fired");
        Ok(())
    }

    pub fn try_reload(&mut self, time: f32) -> Result<(), ReloadError> {
        println!("Try reload");
        self.can_reload(time)?;
        if self.characteristics.reloading_empties_mag {
            self.ammo_loaded = self.ammo_left;
            self.ammo_left -= self.ammo_loaded;
        } else {
            let diff = min(self.characteristics.max_loaded - self.ammo_loaded, self.ammo_left);
            if diff == 0 {
                return Err(ReloadError::FullMag)
            }
            self.ammo_loaded += diff;
            self.ammo_left -= diff;
        }
        self.last_reload_time = Some(time);
        println!("Reloaded");
        Ok(())
    }

    pub fn can_fire(&self, time: f32) -> Result<(), FireError> {
        if let Some(last_fire_time) = self.last_fire_time {
            if time < last_fire_time as f32 + self.characteristics.fire_cd {
                return Err(FireError::FireRate);
            }
        }
        if self.is_empty() {
            return Err(FireError::EmptyMag);
        }
        if self.is_reloading(time) {
            return Err(FireError::Reloading);
        }
        Ok(())
    }

    pub fn can_reload(&self, time: f32) -> Result<(), ReloadError> {
        if self.is_reloading(time) {
            return Err(ReloadError::AlreadyReloading)
        }
        if self.is_full() {
            return Err(ReloadError::FullMag)
        }
        if self.ammo_left == 0 {
            return Err(ReloadError::NoMoreAmmo)
        }
        Ok(())
    }

    pub fn is_reloading(&self, time: f32) -> bool {
        if let Some(last_reload_time) = self.last_reload_time {
            return time < last_reload_time as f32 + self.characteristics.reload_time;
        }
        false
    }

    pub fn is_empty(&self) -> bool {
        self.ammo_loaded == 0
    }

    pub fn is_full(&self) -> bool {
        self.ammo_loaded == self.characteristics.max_loaded
    }

    pub fn get_equip_time(&self) -> f32 {
        self.characteristics.equip_time
    }

    pub fn get_unequip_time(&self) -> f32 {
        self.characteristics.unequip_time
    }
}

#[derive(Debug)]
pub enum FireError {
    EmptyMag,
    FireRate,
    SwappingWeapons,
    Reloading,
    WeaponUnavailable,
}

#[derive(Debug)]
pub enum ReloadError {
    FullMag,
    AlreadyReloading,
    NoMoreAmmo,
    SwappingWeapons,
    WeaponUnavailable,
}