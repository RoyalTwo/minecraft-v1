use std::f32::consts::PI;

use bevy::{prelude::*, window::CursorGrabMode};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};

#[derive(Component)]
struct IsCamera;
#[derive(Component)]
struct BlockID(i32);
#[derive(Bundle)]
struct BlockBundle {
    id: BlockID,

    #[bundle]
    pbr: PbrBundle,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb(0.47, 0.91, 1.0)))
        .add_startup_system(setup)
        .add_system(lock_mouse)
        .add_plugin(FlyCameraPlugin)
        .add_system(click_ray_cast)
        .run();
}

fn setup(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            ..Default::default()
        },
        IsCamera,
        FlyCamera {
            sensitivity: 10.0,
            ..FlyCamera::default()
        },
    ));
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 100.0, 0.0),
            rotation: Quat::from_rotation_x(3.0 * PI / 2.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let chunk = generate_temp_chunk(meshes, materials, asset_server, (0.0, 0.0));
    spawn_temp_chunk(chunk, commands);
}

fn lock_mouse(
    mut window: Query<&mut Window>,
    mouse: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
) {
    let mut window = window.single_mut();

    if mouse.just_pressed(MouseButton::Left) {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    }
}

fn click_ray_cast(
    mouse: Res<Input<MouseButton>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&Transform, With<IsCamera>)>,
    mut commands: Commands,
) {
    let cam = query.single_mut();
    let cam = cam.0;
    let cam_pos = cam.translation;

    let mut ray_transform = Transform::from_xyz(cam_pos.x, cam_pos.y, cam_pos.z);
    ray_transform.rotation = cam.rotation;
    if mouse.just_pressed(MouseButton::Left) {
        let ray_phys = PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 2.0 })),
            material: materials.add(StandardMaterial {
                base_color: Color::Rgba {
                    red: 1.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 1.0,
                },
                ..Default::default()
            }),
            transform: ray_transform,
            ..Default::default()
        };
        commands.spawn(ray_phys);
        info!("Clicked");
    }
}

fn generate_temp_chunk(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    offsets: (f32, f32),
) -> Vec<BlockBundle> {
    const CHUNK_WIDTH: i32 = 16;
    const CHUNK_LENGTH: i32 = 16;

    let grass_texture = asset_server.load("grass_top.png");
    let mut blocks: Vec<BlockBundle> = Vec::new();

    for x in 0..CHUNK_LENGTH {
        for z in 0..CHUNK_WIDTH {
            let new_block = BlockBundle {
                id: BlockID(1),
                pbr: PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                    material: materials.add(StandardMaterial {
                        base_color_texture: Some(grass_texture.clone()),
                        ..Default::default()
                    }),
                    transform: Transform::from_xyz(
                        offsets.0 + (x as f32),
                        0.0,
                        offsets.1 + (z as f32),
                    ),
                    ..Default::default()
                },
            };

            blocks.push(new_block);
        }
    }
    return blocks;
}

fn spawn_temp_chunk(blocks: Vec<BlockBundle>, mut commands: Commands) {
    for block in blocks {
        commands.spawn(block);
    }
}
