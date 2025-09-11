use bevy::{
    //color::palettes::css::{ORANGE, ORANGE_RED},
    //ecs::{query, system::entity_command::insert},
    prelude::*,
};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
//use std::f32::consts::PI;

//use bevy::input::common_conditions::*;
use bevy_flycam::prelude::*;
use bevy_framepace::*;
//use grid::*;
use iyes_perf_ui::prelude::*;

use std::collections::VecDeque; //Stack?

//globals
static PI: f32 = 3.14159265;
static PI_HALF: f32 = 1.57079632; //represent radian ops as fp multiplication instead of division 

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
        .insert_resource(ClearColor(Color::BLACK))
        //Update functions
        .add_systems(Update, move_player.run_if(any_with_component::<Moving>))
        .add_systems(Update, turn_player.run_if(any_with_component::<Rotating>))
        .add_systems(Update, keyboard_input.run_if(has_arrow_input))
        .run();
}

#[derive(Component)]
struct Moving {
    distance: f32,
    direction: Direction,
}

#[derive(Component)]
struct ProcessingInput;

//#[derive(Component)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Direction {
    //components have directions, but the direction is not a component itself
    Left = 2,
    Right = 0,
    Up = 1,
    Down = 3,
}

#[derive(Component)] //radian direction
struct Rotating {
    direction: f32,
}

#[derive(Component)]
struct Player {
    direction: f32,
}

fn has_arrow_input(keys: Res<ButtonInput<KeyCode>>) -> bool {
    keys.pressed(KeyCode::ArrowLeft)
        || keys.pressed(KeyCode::ArrowRight)
        || keys.pressed(KeyCode::ArrowUp)
        || keys.pressed(KeyCode::ArrowDown)
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
        info!("processing input");
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
            let dir_rad = (dir as i32 as f32) * PI_HALF;
            info!("set dir {:?}", dir_rad);

            commands.entity(entity).insert(Moving {
                direction: dir,
                distance: 1.0,
            });

            if dir_rad != player.direction {
                //player.direction still holds original dir
                commands
                    .entity(entity)
                    .insert(Rotating { direction: dir_rad });
                //player.direction = dir; //new direction post-rotate
            }
        }
    }
}

/*fn direction_to_radian(direction: Direction) -> f32 {
    match direction {
        Direction::Left => {
            info!("hi");
        }
        Direction::Right = {

        }

    }
}*/

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut settings: ResMut<bevy_framepace::FramepaceSettings>, //mut frame_settings: FramepaceSettings,
) {
    settings.limiter = Limiter::from_framerate(120.0);

    //perf counter
    commands.spawn(PerfUiDefaultEntries::default());

    // square base
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(16.0, 16.0))), //x,y
        MeshMaterial3d(materials.add(Color::srgb_u8(255, 255, 255))),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)), //x,
    ));

    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1., 1., 1.))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(-7.5, 0.5, 8.5),
        //Moving { distance: 1.0 },
        //Direction::Up,
        Player { direction: PI_HALF },
    ));
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(16.0, 16.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(255, 0, 0))),
        Transform::from_translation(vec3(0.0, 8.0, -8.0)),
    ));
    //boss
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
        //let mut deltatime = time.delta_secs();
        let tick_rate = time.delta_secs() / 0.6; //0.8 updates per second

        //calculate translation
        let translate_axis: &mut f32;
        let mut sign: f32 = 1.0;
        match moving.direction {
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
    let rot_speed = 1.5 * PI; // one π radian per second
    let rot_tick = time.delta_secs() * rot_speed;

    for (entity, mut player, mut transform, rotation) in &mut query {
        // normalize angle difference into (-π, π]
        let mut delta = rotation.direction - player.direction;
        delta = (delta + PI).rem_euclid(2.0 * PI) - PI;

        if delta.abs() <= rot_tick {
            // finish rotation exactly
            transform.rotate_y(delta);
            player.direction = rotation.direction;
            commands.entity(entity).remove::<Rotating>();
            info!("Finished rotation to {:?}", rotation.direction);
        } else {
            // step towards target
            let step = rot_tick.copysign(delta); // use delta's sign
            transform.rotate_y(step);
            player.direction += step;
        }
    }
}
