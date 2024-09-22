use avian3d::{
    math::{
        Quaternion,
        Vector,
    },
    prelude::{
        Collider,
        LockedAxes,
        PhysicsSet,
        RigidBody,
        ShapeCaster,
    },
};
use bevy::{
    color::palettes::css, math::VectorSpace, prelude::*
};
use bevy_steam_p2p::{NetworkIdentity, NetworkedTransform};
use camera_rig::TrackedEntity;
use input::PlayerActions;
use leafwing_input_manager::InputManagerBundle;
use movement::Gravity;

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
    mut commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
    network_identity: NetworkIdentity
) {
    commands.spawn((
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
    ));
}
