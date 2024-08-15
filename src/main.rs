use std::f32::{consts::*, NAN};

use bevy::{math::{NormedVectorSpace, VectorSpace}, prelude::*, render::mesh::{self, skinning::SkinnedMesh}};

#[derive(Component)]
struct Movable;

#[derive(Component)]
struct Target;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(AmbientLight {
            brightness: 750.0,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, joint_animation)
        .add_systems(Update, movable)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,) {
    // Create a camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 5., -2.0)
            .looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
        ..default()
    });

    let target = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.1, 0.1, 0.1)),
            material: materials.add(Color::srgb_u8(124, 144, 255)),
            transform: Transform::from_xyz(0.85, 0.0, 0.0),
            ..default()
        },
      //  Movable,
        Target,
    )).id();

    // Spawn the first scene in `models/SimpleSkin/SimpleSkin.gltf`
    commands.spawn((SceneBundle {
        scene: asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("leg/leg.glb")),
        transform: Transform::from_xyz(0.0, 0., 0.0),
        ..default()
    }, Movable));

    commands.spawn(SceneBundle {
        scene: asset_server.load("map/map.glb#Scene0"),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });

    

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

}

fn joint_animation(
    time: Res<Time>,
    parent_query: Query<(Entity, &Parent, &SkinnedMesh)>,
    children_query: Query<(&Children)>,
    movable_query: Query<Entity, With<Target>>,
    mut transform_query: Query<&mut Transform>,
    mut gizmos: Gizmos,

) {
    let Ok((target_entity)) = movable_query.get_single() else { return; };
    // Iter skinned mesh entity
    for (entity, skinned_mesh_parent, skinned_mesh) in &parent_query {
        let Ok([mut t0, mut t1, target, transform]) = transform_query.get_many_mut([skinned_mesh.joints[0], skinned_mesh.joints[1], target_entity, entity]) else { println!("fuck"); continue; };
        let dir = (target.translation - transform.translation);
        //let y = (-transform.local_x()).xz().angle_between(dir.xz());
        let (y, z) = calc_angles(&transform, dir);
        println!("{}", y.to_degrees());
       //gizmos.ray(Vec3::ZERO, dir, Color::srgb(1., 0., 0.));
       //gizmos.ray(Vec3::ZERO, transform.forward().as_vec3(), Color::srgb(0., 1., 0.));
       gizmos.ray(Vec3::ZERO, transform.local_x().as_vec3(), Color::srgb(0., 1., 0.));
       gizmos.ray(Vec3::ZERO, dir.xy().extend(0.), Color::srgb(0., 1., 0.));
       //gizmos.cuboid(Transform::from_translation(target.translation).with_scale(Vec3::splat(0.1)), Color::srgb(1., 0., 0.));
        let d_a: f32 = t0.translation.distance(t1.translation);
        let d_b: f32 = t0.translation.distance(t1.translation);
        let mut d_c = dir.length();
        if (d_c > d_a + d_c) {
            d_c = d_a + d_c;
        }
       // println!("{} {}", d_c, t0.translation.distance(target.translation));
        //let mut a = ((d_c.powf(2.) + d_b.powf(2.) - d_a.powf(2.)) / 2. * d_c * d_b).acos();
        let mut a = calc_necessary_angle(d_b, d_c, d_a);
        let mut b: f32 = calc_necessary_angle(d_a, d_b, d_c);
       // let mut b = calc_necessary_angle(d_a, d_b, d_c);
        //let mut a = ((180_f32).to_radians() - b) / 2.;
      //  println!("{} {} {} {} / {} {} {}", a, b, a.to_degrees(), b.to_degrees(), d_a, d_b, d_c);

       // let mut a = calc_necessary_angle(d_a, d_b, d_c);
        //let mut b = calc_necessary_angle(d_a, d_b, d_c);
        if a.is_nan(){
            a = 0.;
        }
        if b.is_nan() {
           // println!("Too far");
            b = PI;
        }
        a = PI/2. - a;
        b = PI - b;
       // b *= 2.;
        //b = PI/2. - b;
        //println!("{} {} {}", a.to_degrees(), b.to_degrees(), d_c);
        //println!("{}", y.to_degrees());
        //t0.rotation = Quat::from_euler(EulerRot::XYZ, 0., 0., 0.);
        t0.rotation = Quat::from_euler(EulerRot::XYZ, 0.,-y, a - z);
        t1.rotation = Quat::from_rotation_z(b);
    }
}

fn calc_angles(transform: &Transform, dir: Vec3) -> (f32, f32) {
    let y = (-transform.local_x()).xz().angle_between(dir.xz());
    let mut inbetween = dir;
    inbetween.y = 0.;
//    let z = inbetween.angle_between(dir);
    let mut z = dir.angle_between(inbetween);
    if (dir.y < 0.) {
        z = -z;
    }
   // let z = (transform.local_x()).xy().angle_between(dir.xy());
    return (y, z);
}

fn calc_necessary_angle(a: f32, b: f32, c: f32) -> f32 {
    let top_part = a.powf(2.) + b.powf(2.) - c.powf(2.);
    let bottom_part = 2. * a * b;
    let result = (top_part / bottom_part).acos();
    //println!("{}, {}, {}", top_part, bottom_part, result);
    return result;
}

fn movable(
    mut transform_query: Query<&mut Transform, With<Movable>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    // Iter skinned mesh entity
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