use avian3d::{
    math::{
        Quaternion,
        Vector,
    }, prelude::{
        Collider,
        LockedAxes,
        PhysicsSet,
        RigidBody,
        ShapeCaster,
    }
};
use bevy::{
    color::palettes::css, math::VectorSpace, prelude::*, render::view::visibility, utils::HashMap
};
use bevy_steam_p2p::{networked_transform::NetworkedTransform, NetworkIdentity, SteamP2PClient };
use camera_rig::{RiggedCamera, TrackedEntity};
use input::PlayerActions;
use leafwing_input_manager::InputManagerBundle;
use movement::Gravity;

use crate::{weapon_system::{auxiliary::{weapon_networking::NetworkedWeaponSystem, weapon_raycaster::WeaponRaycaster}, visuals::gltf::WeaponVisualsManagerGltf, weapon::{Weapon, WeaponCharacteristics}, weapon_inventory::WeaponInventory, WeaponSystem}, Crosshair};

mod camera_rig;
mod input;
mod kinematic_controller;
mod movement;
mod weapon;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        camera_rig::plugin,
        movement::plugin,
        input::plugin,
        kinematic_controller::plugin,
        weapon::plugin,
    ));
    app.configure_sets(
        FixedUpdate,
        CharacterControllerSet::Input,
    );
    app.configure_sets(
        PostUpdate,
        CharacterControllerSet::CameraSync
            .after(PhysicsSet::Sync)
            .before(TransformSystem::TransformPropagate)
    );
}

#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum CharacterControllerSet {
    Input,
    CameraSync,
}

#[derive(Bundle)]
pub struct CharacterControllerBundle {
    pub tracked_entity: TrackedEntity,
    pub current_player: CurrentPlayer,
    pub player: Player,
    pub input: InputManagerBundle<PlayerActions>,
    pub gravity: Gravity,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub ground_caster: ShapeCaster,
    pub kinematic_controller: kinematic_controller::KinematicCharacterController,
    pub kcc_grounded: kinematic_controller::KCCGrounded,
    pub kcc_floor_detection: kinematic_controller::KCCFloorDetection,
}

impl Default for CharacterControllerBundle {
    fn default() -> Self {
        Self {
            tracked_entity: TrackedEntity(Vec3::new(0.0, 0.5, 0.0)),
            current_player: CurrentPlayer,
            player: Player,
            input: InputManagerBundle::with_map(input::input_map()),
            gravity: Gravity::default(),
            rigid_body: RigidBody::Kinematic,
            collider: Capsule3d::new(0.4, 0.8).into(),
            ground_caster: ShapeCaster::new(
                Capsule3d::new(0.4, 0.8),
                Vector::ZERO,
                Quaternion::default(),
                Dir3::NEG_Y,
            )
            .with_max_time_of_impact(0.1),
            kinematic_controller: kinematic_controller::KinematicCharacterController::default(),
            kcc_grounded: kinematic_controller::KCCGrounded::default(),
            kcc_floor_detection: kinematic_controller::KCCFloorDetection::default(),
        }
    }
}

// Signifies our current player.
#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct CurrentPlayer;

// Signifies that this entity is a player.
#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct Player;

pub fn spawn_test_character(
    mut client: &mut ResMut<SteamP2PClient>,
    mut commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
    network_identity: NetworkIdentity,
    mut crosshair_query: &mut Query<(&mut Style, &mut Visibility, Option<&Crosshair>)>
) {
    let id = network_identity.owner_id.clone();
    let character = commands.spawn((
        CharacterControllerBundle::default(),
        PbrBundle {
            mesh: meshes.add(Capsule3d { radius: 0.4, half_length: 0.4 }),
            material: materials.add(Color::from(css::DARK_CYAN)),
            transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            ..Default::default()
        },
        NetworkedTransform { synced: true, target: Vec3::ZERO },
        network_identity,
        LockedAxes::ROTATION_LOCKED,
        Name::new("CurrentPlayer"),
        WeaponSystem {
            inventory: WeaponInventory::new(vec![
                Weapon::new(
                    WeaponCharacteristics {
                        damage: 40,
                        max_loaded: 12,
                        max_ammo: 250,
                        fire_cd: 0.2,
                        reload_time: 2.,
                        equip_time: 0.2,
                        unequip_time: 0.1,
                        reloading_empties_mag: false
                    },
                    50, 
                    true,
                    "glock" 
                )   
            ]),
        },
        NetworkedWeaponSystem
    )).id();
    let mut match_list = HashMap::new();
    match_list.insert("glock".to_string(), "weapons/glock/glock.glb".to_string());
    if client.id == id {
        commands.spawn((
            RiggedCamera { tracked: character },
            Camera3dBundle {
                // Adjust our rotation so we're looking backwards on spawn
                transform: Transform::from_xyz(0.0, 0.0, 0.0)
                    .looking_at(Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 0.0)).with_scale(Vec3::ONE * 15.),
                camera: Camera {
                    clear_color: ClearColorConfig::Custom(Color::linear_rgb(0.384, 0.71, 0.949)),
                    ..Default::default()
                },
                projection: Projection::Perspective(PerspectiveProjection {
                    near: 0.01,
                    ..default()
                }),
                ..Default::default()
            },
            WeaponVisualsManagerGltf {
                match_list,
                system: character,
            },
            WeaponRaycaster {
                system: character
            },
            Name::new("WeaponCamera")
        ));
    } else {
        commands.spawn((
            SpatialBundle {
                ..default()
            },
            WeaponVisualsManagerGltf {
                match_list,
                system: character,
            },
            WeaponRaycaster {
                system: character
            },
            Name::new("WeaponEmpty"),
            RiggedCamera { tracked: character },
        ));
    }
    
    for (mut style, mut visibility, maybe_crosshair) in crosshair_query.iter_mut() {
        style.set_changed();
        if let Some(_) = maybe_crosshair {
            *visibility = Visibility::Inherited
        } 
    }
}
