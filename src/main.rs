use std::{env, f32::{consts::*, NAN}};
use bevy::{color::palettes, gltf::GltfMesh, math::{NormedVectorSpace, VectorSpace}, prelude::*, render::mesh::{self, skinning::SkinnedMesh}};
use bevy_mod_raycast::prelude::NoBackfaceCulling;
use leg::{IKLeg, LegCreature, LegCreatureVisual, LegPlugin, LegSide};
use rand::distributions::Standard;
use spider::{spawn_spider, spawn_test_arm};
use vleue_navigator::{NavMesh, VleueNavigatorPlugin};
use IKArm::{IKArmPlugin, IKArmTarget};

mod IKArm;
mod leg;
mod spider;

#[derive(Component)]
struct Movable;

#[derive(Component)]
struct MultiPosCamera {
    positions: Vec<(Vec3, Vec3)>,
    index: i32
}

#[derive(Resource)]
struct MapLoader {
    handle: Option<Handle<Gltf>>
}

#[derive(Component)]
struct GroundMarker;

const HANDLE_TRIMESH_OPTIMIZED: Handle<NavMesh> = Handle::weak_from_u128(0);

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    App::new()
        .add_plugins((DefaultPlugins, VleueNavigatorPlugin))
        .add_plugins((IKArmPlugin, LegPlugin))
        .insert_resource(AmbientLight {
            brightness: 750.0,
            ..default()
        })
        .insert_resource(MapLoader {handle: None})
        .add_systems(Startup, (setup, ).chain())
        .add_systems(Update, (movable, multi_pos_camera, spawn_map))
       // .observe(modify_meshes)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut map: ResMut<MapLoader>,
) {
    // Create a camera
    commands.spawn((Camera3dBundle {
            transform: Transform::from_xyz(-7.0, 7., -7.0)
                .looking_at(Vec3::new(0.0, 0., 0.0), Vec3::Y),
            ..default()
        },
        MultiPosCamera { 
            positions: vec![
                (Vec3::new(-7.0, 7., -7.0), Vec3::new(0.0, 0., 0.0)),
                (Vec3::new(0.0, 10., 0.0), Vec3::new(0.0, 0., 0.0))
            ],
            index: 0,
        }
    ));

    spawn_spider(&mut commands, &asset_server, &mut meshes, &mut materials);
    //spawn_test_arm(&mut commands, &asset_server, &mut meshes, &mut materials);
    /* 
    commands.spawn((
        SceneBundle {
        scene: asset_server.load("map/map.glb#Scene0"),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
        },
        GroundMarker,
    ));
    */
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    map.handle = Some(asset_server.load("map/map_slopes.glb"));


}

fn spawn_map(
    mut commands: Commands,
    mut map: ResMut<MapLoader>,
    assets_gltf: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut navmeshes: ResMut<Assets<NavMesh>>,
) {
    // if the GLTF has loaded, we can navigate its contents
    let Some(handle) = &map.handle else { return; };
    if let Some(gltf) = assets_gltf.get(handle) {
        // spawn the first scene in the file
        commands.spawn(SceneBundle {
            scene: gltf.scenes[0].clone(),
            visibility: Visibility::Hidden,
            ..Default::default()
        });
     
        let Some(gltf_mesh) = gltf_meshes.get(&gltf.named_meshes["Ground"]) else {return;};
        let Some(mesh) = meshes.get(&gltf_mesh.primitives[0].mesh) else {return;};
        let navmesh = NavMesh::from_bevy_mesh(mesh);

        let mut material: StandardMaterial = Color::Srgba(palettes::css::ANTIQUE_WHITE).into();
        material.unlit = true;

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(navmesh.to_wireframe_mesh()),
                material: materials.add(material),
                transform: Transform::from_xyz(0.0, 0.2, 0.0),
                ..Default::default()
            },
            Name::new("Ground")
        ));
        navmeshes.insert(&HANDLE_TRIMESH_OPTIMIZED, navmesh);
        map.handle = None;
    }
}

fn movable(
    mut transform_query: Query<&mut Transform, With<Movable>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for (mut movable_transform) in transform_query.iter_mut() {
        let mut vec = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) {
            vec.z += 1.0
        }
        if keys.pressed(KeyCode::KeyS) {
            vec.z -= 1.0
        }
        if keys.pressed(KeyCode::KeyD) {
            vec.x -= 1.0
        }
        if keys.pressed(KeyCode::KeyA) {
            vec.x += 1.0
        }
        if keys.pressed(KeyCode::KeyQ) {
            vec.y += 1.0
        }
        if keys.pressed(KeyCode::KeyE) {
            vec.y -= 1.0
        }
        movable_transform.translation += vec * 0.01;
    }
}

fn multi_pos_camera(
    mut camera_query: Query<(&mut Transform, &mut MultiPosCamera)>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for (mut transform, mut multi_pos) in camera_query.iter_mut() {
        if keys.just_pressed(KeyCode::ArrowLeft) {
            multi_pos.index -= 1;
        }
        else if keys.just_pressed(KeyCode::ArrowRight) {
            multi_pos.index += 1;
        }
        multi_pos.index = multi_pos.index.rem_euclid(multi_pos.positions.len() as i32);
        //multi_pos.index = multi_pos.index % (multi_pos.positions.len() as i32);
        let (position, target) = multi_pos.positions[multi_pos.index as usize];
        *transform = Transform::from_translation(position).looking_at(target, Vec3::Y);
    }
}