//use std::ptr::null;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

//use bevy::input::common_conditions::*;
use bevy_flycam::prelude::*;
//use bevy_framepace::*;
//use grid::*;
use iyes_perf_ui::prelude::*;

#[bevy_main]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        //Egui Inspector
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        //Flycam
        .add_plugins(PlayerPlugin)
        //FPS and diagnostics
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        .add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin)
        .add_plugins(PerfUiPlugin)
        .add_plugins(bevy_framepace::FramepacePlugin)
        //Bg color
        //.insert_resource(ClearColor(Color::rgb(0.9, 0.3, 0.6)))
        //Update functions
        .add_systems(Update, move_step)
        .add_systems(Update, keyboard_input)
        .run();
}

#[derive(Component)]
struct Moving {
    distance: f32,
}

#[derive(Component)]
struct Rotating;

#[derive(Component)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Component)]
struct Player;
//Query<(Entity, &mut Transform), With<Moving>>,
fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    query: Query<Entity, (With<Player>, Without<Moving>)>,
    mut commands: Commands,
) {
    for entity in &query {
        if keys.pressed(KeyCode::ArrowLeft) {
            commands
                .entity(entity)
                .insert((Moving { distance: 1.0 }, Direction::Left));
        } else if keys.pressed(KeyCode::ArrowRight) {
            commands
                .entity(entity)
                .insert((Moving { distance: 1.0 }, Direction::Right));
        } else if keys.pressed(KeyCode::ArrowUp) {
            commands
                .entity(entity)
                .insert((Moving { distance: 1.0 }, Direction::Up));
        } else if keys.pressed(KeyCode::ArrowDown) {
            commands
                .entity(entity)
                .insert((Moving { distance: 1.0 }, Direction::Down));
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    //  let mut grid = grid![[0,0,0] [0,0,0] [0,0,0]];

    //perf counter
    commands.spawn(PerfUiDefaultEntries::default());

    // square base
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(16.0, 16.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));

    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1., 1., 1.))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.6, 0.0),
        Moving { distance: 1.0 },
        Direction::Up,
        Rotating,
        Player,
    ));

    //boss
}

fn move_step(
    time: Res<Time>,
    mut commands: Commands,
    //mut query: Query<(Entity, &mut Transform), With<Moving>>,
    mut query: Query<(Entity, &mut Transform, &mut Moving, &Direction)>,
) {
    for (entity, mut transform, mut moving, direction) in &mut query {
        //transform.translation.x += time.delta_secs();

        //speed indirectly represented by deltatime. doubling deltatime moves twice as fast
        let mut deltatime = time.delta_secs();

        //2x speed
        deltatime *= 2.0;

        //calculate translation
        let translate_axis: &mut f32;
        let mut sign: f32 = 1.0;
        match direction {
            Direction::Left => {
                translate_axis = &mut transform.translation.x;
                sign = -1.0;
                //deltatime = -deltatime;
            }
            Direction::Right => {
                translate_axis = &mut transform.translation.x;
            }
            Direction::Up => {
                translate_axis = &mut transform.translation.z;
                sign = -1.0;
                //deltatime = -deltatime;
            }
            Direction::Down => {
                translate_axis = &mut transform.translation.z;
            } // _ => panic!("unrecognized direction!"), //translate_axis = &mut transform.translation.x,
        }

        //perform translation
        if moving.distance > deltatime {
            *translate_axis += deltatime * sign;

            //transform.translation.x += deltatime;
            moving.distance -= deltatime;
        } else {
            *translate_axis += (deltatime * sign) - moving.distance;
            //transform.translation.x += (deltatime - moving.distance);
            //moving.distance = 0.0; not needed
            commands.entity(entity).remove::<Moving>();
        }
        //info!("{}", transform.translation.x);
        //if transform.translation.x >= 4.5 {
        //   commands.entity(entity).remove::<Moving>();
        //}
    }
}
