use bevy::prelude::*;

//globals
#[derive(Resource)]
pub struct Animations {
    pub graph_handle: Handle<AnimationGraph>,
    pub animations: Vec<AnimationNodeIndex>,
}

#[derive(Component)]
pub struct Moving {
    pub distance: f32,
    pub direction: Direction,
}

#[derive(Component)]
pub struct ProcessingInput;

//#[derive(Component)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    //components have directions, but the direction is not a component itself
    Left = 2,
    Right = 0,
    Up = 1,
    Down = 3,
}

#[derive(Component)] //radian direction
pub struct Rotating {
    pub direction: f32,
}

#[derive(Component)]
pub struct Player {
    pub direction: f32,
}
