use std::f32::consts::PI;

use bevy::{prelude::*, transform::commands, window::CursorGrabMode};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy_mod_raycast::{
    DefaultPluginState, DefaultRaycastingPlugin, Intersection, RaycastMesh, RaycastMethod,
    RaycastSource, RaycastSystem,
};

#[derive(Component)]
struct IsCamera;
#[derive(Component)]
struct IsBlock;
#[derive(Component)]
struct BlockID(i32);
#[derive(Bundle)]
struct BlockBundle {
    id: BlockID,

    #[bundle]
    pbr: PbrBundle,
}
#[derive(Clone, Reflect)]
struct MyRaycastSet;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(DefaultRaycastingPlugin::<MyRaycastSet>::default())
        .add_system(
            update_raycast_with_cursor
                .in_base_set(CoreSet::First)
                .before(RaycastSystem::BuildRays::<MyRaycastSet>),
        )
        .insert_resource(ClearColor(Color::rgb(0.47, 0.91, 1.0)))
        .add_startup_system(setup)
        .add_system(lock_mouse)
        .add_system(break_block)
        .add_plugin(FlyCameraPlugin)
        .run();
}

fn setup(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(DefaultPluginState::<MyRaycastSet>::default().with_debug_cursor());
    commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, 2.0, 0.0),
                ..Default::default()
            },
            IsCamera,
            FlyCamera {
                sensitivity: 10.0,
                ..FlyCamera::default()
            },
        ))
        .insert(RaycastSource::<MyRaycastSet>::new()); // Make this mesh ray cast-able
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
        let window_dimensions = Vec2 {
            x: window.width() / 2.0,
            y: window.height() / 2.0,
        };
        window.set_cursor_position(Some(window_dimensions));
        // window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    }
}

fn update_raycast_with_cursor(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<&mut RaycastSource<MyRaycastSet>>,
) {
    // Grab the most recent cursor event if it exists:
    let cursor_position = match cursor.iter().last() {
        Some(cursor_moved) => cursor_moved.position,
        None => return,
    };

    for mut pick_source in &mut query {
        pick_source.cast_method = RaycastMethod::Screenspace(cursor_position);
    }
}

fn break_block(
    mouse: Res<Input<MouseButton>>,
    query: Query<&Intersection<MyRaycastSet>>,
    blocks: Query<(&Transform, Entity), With<IsBlock>>,
    mut commands: Commands,
) {
    if mouse.just_pressed(MouseButton::Left) {
        for intersection in &query {
            if let Some(pos) = intersection.position() {
                for (transform, entity) in blocks.iter() {
                    let block_pos = transform.translation;

                    let inter_coords = Vec3::new(pos.x + 0.5, pos.y - 0.5, pos.z + 0.5);
                    let inter_after_floor = (
                        inter_coords.x.floor(),
                        inter_coords.y.floor(),
                        inter_coords.z.floor(),
                    );
                    if inter_after_floor.0 == block_pos.x
                        && inter_after_floor.1 == block_pos.y
                        && inter_after_floor.2 == block_pos.z
                    {
                        info!("Block: {:?}", block_pos);
                        commands.entity(entity).despawn();
                    } else {
                        warn!(
                            "E Inter {:?} Block Inter {:?}",
                            inter_after_floor, block_pos
                        );
                    }
                }
            }
        }
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
        commands
            .spawn((block, IsBlock))
            .insert(RaycastMesh::<MyRaycastSet>::default()); // Make this mesh ray cast-able
    }
}
