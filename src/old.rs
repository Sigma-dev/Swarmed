use bevy::{
    pbr::CascadeShadowConfigBuilder, prelude::*, render::mesh::skinning::SkinnedMesh
};
use bevy::prelude::Gltf::GltfAssetLabel;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, joint_animation)
        .run();
}


fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 5., 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn(SceneBundle {
        scene: asset_server.load("map/map.glb#Scene0"),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });

    commands.spawn(SceneBundle {
        scene: asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("leg/leg.glb")),
        transform: Transform::from_xyz(0., 1.0, 0.),
        ..default()
    });


    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-3.14 / 4.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 10.0,
            ..default()
        }
        .into(),
        ..default()
    });
}

fn joint_animation(
    time: Res<Time>,
    parent_query: Query<&Parent, With<SkinnedMesh>>,
    children_query: Query<&Children>,
    mut transform_query: Query<&mut Transform>,
) {
    // Iter skinned mesh entity
    for skinned_mesh_parent in &parent_query {
        // Mesh node is the parent of the skinned mesh entity.
        let mesh_node_entity = skinned_mesh_parent.get();
        // Get `Children` in the mesh node.
        let mesh_node_children = children_query.get(mesh_node_entity).unwrap();

        // First joint is the second child of the mesh node.
        let first_joint_entity = mesh_node_children[0];
        // Get `Children` in the first joint.
        let first_joint_children = children_query.get(first_joint_entity).unwrap();

        // Second joint is the first child of the first joint.
        let second_joint_entity = first_joint_children[0];
        // Get `Transform` in the second joint.
        let mut second_joint_transform = transform_query.get_mut(second_joint_entity).unwrap();

        second_joint_transform.rotation =
            Quat::from_rotation_z(1.6 * time.elapsed_seconds().sin());
    }
}