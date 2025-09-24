#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![forbid(unsafe_code)]

use bevy::{
    color::palettes::css::RED,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    scene::SceneInstanceReady,
};
use bevy_ecs_tilemap::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use bevy_flycam::prelude::*;
use bevy_framepace::*;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin};
use bevy_inspector_egui::{DefaultInspectorConfigPlugin, quick::WorldInspectorPlugin};
use bevy_mod_outline::{
    AsyncSceneInheritOutline, AutoGenerateOutlineNormalsPlugin, OutlinePlugin, OutlineVolume,
};
use iyes_perf_ui::prelude::*;
use std::collections::VecDeque; //Stack?
use std::{f32::consts::*, time::Duration};

mod components;
use components::*;

const PLAYER_GLTF_PATH: &str = "player/player.glb";
// const SHADER_ASSET_PATH: &str = "shaders/outline_mesh.wgsl";
const SHADER_ASSET_PATH: &str = "shaders/animate_shader.wgsl";
#[bevy_main]
fn main() {
    App::new()
        //.add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_plugins((
            DefaultPlugins,
            PlayerPlugin,
            EguiPlugin::default(),
            //WorldInspectorPlugin::new(),
            bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
            bevy::diagnostic::EntityCountDiagnosticsPlugin,
            bevy::diagnostic::SystemInformationDiagnosticsPlugin,
            bevy::render::diagnostic::RenderDiagnosticsPlugin,
            PerfUiPlugin,                    //PerfUI
            bevy_framepace::FramepacePlugin, //FPS limiter
            InfiniteGridPlugin,
            OutlinePlugin, //Mesh outlining
            AutoGenerateOutlineNormalsPlugin::default(),
            MaterialPlugin::<CustomMaterial>::default(),
            MeshPickingPlugin,
            TilemapPlugin,
        ))
        .add_plugins(WorldInspectorPlugin::new())
        //.add_plugins(ImagePlugin)
        //Bg color
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Update, setup_scene_once_loaded)
        //Update functions
        .add_systems(Update, move_player.run_if(any_with_component::<Moving>))
        .add_systems(Update, turn_player.run_if(any_with_component::<Rotating>))
        .add_systems(Update, keyboard_input.run_if(has_arrow_input))
        .run();
}

fn has_arrow_input(keys: Res<ButtonInput<KeyCode>>) -> bool {
    keys.pressed(KeyCode::ArrowLeft)
        || keys.pressed(KeyCode::ArrowRight)
        || keys.pressed(KeyCode::ArrowUp)
        || keys.pressed(KeyCode::ArrowDown)
}

fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>, //this queries the
                                                                                //animation player entities, not the parents..
) {
    for (entity, mut player) in &mut players {
        info!("new animation starting loop");
        let mut transitions = AnimationTransitions::new();
        // Make sure to start the animation via the `AnimationTransitions`
        // component. The `AnimationTransitions` component wants to manage all
        // the animations and will get confused if the animations are started
        // directly via the `AnimationPlayer`.
        transitions
            .play(&mut player, animations.animations[0], Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(AnimationGraphHandle(animations.graph_handle.clone()))
            .insert(transitions);
    }
}

//Query<(Entity, &mut Transform), With<Moving>>,
fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, &mut Player), Without<Moving>>, //TODO: not precise
    mut commands: Commands,
) {
    for (entity, mut player) in &mut query {
        //player
        //
        // info!("processing input");
        let mut arrow_pressed: Option<Direction> = None;
        if keys.pressed(KeyCode::ArrowLeft) {
            arrow_pressed = Some(Direction::Left);

            //info!("since last run {:?}", time.elapsed_secs());
        } else if keys.pressed(KeyCode::ArrowRight) {
            arrow_pressed = Some(Direction::Right);
        } else if keys.pressed(KeyCode::ArrowUp) {
            arrow_pressed = Some(Direction::Up);
        } else if keys.pressed(KeyCode::ArrowDown) {
            arrow_pressed = Some(Direction::Down);
        }
        //should player move

        if let Some(dir) = arrow_pressed {
            let dir_rad = (dir as i32 as f32) * FRAC_PI_2;
            //info!("set dir {:?}", dir_rad);

            commands.entity(entity).insert(Moving {
                direction: dir,
                distance: 1.0,
            });

            if dir_rad != player.direction {
                //player.direction still holds original dir
                commands
                    .entity(entity)
                    .insert(Rotating { direction: dir_rad });
            }
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CustomMaterial {}

impl Material for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut settings: ResMut<bevy_framepace::FramepaceSettings>, //mut frame_settings: FramepaceSettings,
    asset_server: Res<AssetServer>,                          //asset server
    mut graphs: ResMut<Assets<AnimationGraph>>,              //animation graphs
) {
    //Game settings
    settings.limiter = Limiter::from_framerate(120.0);

    //Debug and perf
    commands.spawn(PerfUiDefaultEntries::default());

    //commands.spawn(Camera2d::default());

    //tilemap
    let map_size = TilemapSize { x: 8, y: 8 };
    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size: TilemapGridSize = tile_size.into();
    let map_type = TilemapType::default();

    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);
    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(0),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        anchor: TilemapAnchor::Center,
        visibility: Visibility::Visible,

        ..Default::default()
    });

    //info!("{:?}", &tile_storage.clone());
    // rect base
    // commands.spawn((
    //     Mesh3d(meshes.add(Rectangle::new(16.0, 16.0))), //x,y
    //     MeshMaterial3d(materials.add(Color::srgb_u8(255, 255, 255))),
    //     Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)), //x,
    // ));

    //    cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(CustomMaterial {})),
        Transform::from_xyz(0.0, 0.5, 0.0),
        OutlineVolume {
            visible: true,
            width: 4.0,
            colour: Color::srgb(1.0, 0.5, 0.0),
        },
    ));

    commands.spawn(InfiniteGridBundle::default());

    //commands.spawn((
    //    Mesh3d(meshes.add(Rectangle::new(16.0, 16.0))),
    //    MeshMaterial3d(materials.add(CustomMaterial {})),
    //    Transform::from_translation(vec3(0.0, 8.0, -8.0)),
    //));
    //

    //Light
    // commands.spawn((
    //     PointLight {
    //         intensity: 100.0,
    //         color: RED.into(),
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     Transform::from_xyz(0.0, 8.0, -5.0),
    //     children![(
    //         Mesh3d(meshes.add(Sphere::new(0.2).mesh().uv(32, 18))),
    //         MeshMaterial3d(materials.add(StandardMaterial {
    //             base_color: RED.into(),
    //             emissive: LinearRgba::new(20.0, 0.0, 0.0, 0.0),
    //             ..default()
    //         })),
    //     )],
    // ));

    let (walk_graph, index) = AnimationGraph::from_clip(
        //anim graph for the walk animation
        asset_server.load(GltfAssetLabel::Animation(0).from_asset(PLAYER_GLTF_PATH)),
    );
    // Store the animation graph as an asset.
    let graph_handle = graphs.add(walk_graph); //handle to locate

    //let walk_animation = AnimationToPlay {
    //    graph_handle,
    //    index,
    //};

    commands.insert_resource(Animations {
        //walk animation is resource
        animations: vec![index],
        graph_handle,
    });

    commands
        .spawn((
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(PLAYER_GLTF_PATH))),
            Player {
                direction: FRAC_PI_2,
            },
            Transform {
                translation: Vec3::new(-7.5, 0.5, 8.5),
                rotation: Quat::from_rotation_y(PI),
                scale: Vec3::new(0.5, 0.5, 0.5),
            },
        ))
        .observe(
            |mut trigger: Trigger<Pointer<Click>>, mut commands: Commands| {
                let entity = trigger.target();
                info!("i was clicked!");

                commands.entity(entity).insert((
                    OutlineVolume {
                        visible: true,
                        width: 4.0,
                        colour: Color::srgb(1.0, 0.5, 0.0),
                    },
                    AsyncSceneInheritOutline::default(),
                ));

                trigger.propagate(false);
            },
        );
}

fn move_player(
    time: Res<Time>,
    mut commands: Commands,
    //mut query: Query<(Entity, &mut Transform), With<Moving>>,
    mut query: Query<(Entity, &mut Transform, &mut Moving)>,
) {
    for (entity, mut transform, mut moving) in &mut query {
        //transform.translation.x += time.delta_secs();

        //speed indirectly represented by deltatime. doubling deltatime moves twice as fast

        let tick_rate = time.delta_secs() / 0.4; //0.8 updates per second

        //calculate translation
        let translate_axis: &mut f32;
        let mut sign: f32 = 1.0;
        match moving.direction {
            Direction::Left => {
                translate_axis = &mut transform.translation.x;
                sign = -1.0;
            }
            Direction::Right => {
                translate_axis = &mut transform.translation.x;
            }
            Direction::Up => {
                translate_axis = &mut transform.translation.z;
                sign = -1.0;
            }
            Direction::Down => {
                translate_axis = &mut transform.translation.z;
            }
        }

        //perform translation
        if moving.distance > tick_rate {
            *translate_axis += tick_rate * sign;
            moving.distance -= tick_rate;
        } else {
            *translate_axis += moving.distance * sign; // exact finish
            commands.entity(entity).remove::<Moving>();
        }
    }
}

fn turn_player(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Player, &mut Transform, &Rotating)>,
) {
    let rot_speed = 1.5 * PI; // one pi radian per second
    let rot_tick = time.delta_secs() * rot_speed;

    for (entity, mut player, mut transform, rotation) in &mut query {
        let mut delta = rotation.direction - player.direction;
        delta = (delta + PI).rem_euclid(2.0 * PI) - PI;

        if delta.abs() <= rot_tick {
            // finish rotation exactly
            transform.rotate_y(delta);
            player.direction = rotation.direction;
            commands.entity(entity).remove::<Rotating>();
        } else {
            // step towards target
            let step = rot_tick.copysign(delta); // use delta's sign
            transform.rotate_y(step);
            player.direction += step;
        }
    }
}
