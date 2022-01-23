use bevy::prelude::*;

#[derive(Copy, Clone)]
pub enum Player { Left, Right }

#[derive(Component)]
pub struct Ball;

#[derive(Component)]
pub struct Paddle(pub Player);

#[derive(Component)]
pub struct Moving {
    pub velocity: Vec3
}

#[derive(Copy, Clone)]
pub enum Collider {
    Wall,
    Ball,
    Paddle(Player),
    Goal(Player)
}

#[derive(Component)]
pub struct Colliding {
    pub kind: Collider ,
    pub size: Vec2,
}

#[derive(Component)]
pub struct ScoreText(pub Player);

#[derive(Component)]
pub struct TitleText;

#[derive(Component)]
pub struct GoalText;

#[derive(Component)]
pub struct WinText;

pub enum PongEvent {
    Goal(Player),
}

//       <------- Win <--------------------          
//      /                                  \
// Title -> NewGame -> Ready -> InGame -> Goal
//                        \                /
//                         <---------------
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    Title,
    NewGame,
    Ready,
    InGame,
    Goal,
    Win
}

pub struct Score {
    pub left: u32,
    pub right: u32,
}

