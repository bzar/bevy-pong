use crate::types::*;
use crate::consts::*;
use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision}
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(SystemSet::on_enter(AppState::NewGame).with_system(new_game))
            .add_system_set(SystemSet::on_enter(AppState::Ready).with_system(ready_enter))
            .add_system_set(SystemSet::on_update(AppState::Ready).with_system(ready_update))
            .add_system_set(SystemSet::on_update(AppState::InGame)
                .with_system(move_system)
                .with_system(ball_collide_system)
                .with_system(event_listener_system))
            .add_system_set(SystemSet::on_enter(AppState::Goal).with_system(goal_enter))
            .add_system_set(SystemSet::on_update(AppState::Goal).with_system(goal_update))
            .add_system_set(SystemSet::on_exit(AppState::Goal).with_system(goal_exit))
            .add_system_set(SystemSet::on_enter(AppState::Win).with_system(win_enter))
            .add_system_set(SystemSet::on_update(AppState::Win).with_system(win_update))
            .add_system_set(SystemSet::on_exit(AppState::Win).with_system(win_exit));
    }
}

fn new_game(mut score: ResMut<Score>,
            mut state: ResMut<State<AppState>>,
            mut score_text_query: Query<&mut Text, With<ScoreText>>) {
    // println!("NewGame");
    score.left = 0;
    score.right = 0;
    for mut text in score_text_query.iter_mut() {
        text.sections[0].value = "0".to_string();
    }
    state.set(AppState::Ready).unwrap()
}

fn ready_enter(mut ball_query: Query<(&Ball, &mut Transform)>) {
    // println!("Ready");
    for (_ball, mut transform) in ball_query.iter_mut() {
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;
    }
}

fn ready_update(mut state: ResMut<State<AppState>>,
                time: Res<Time>,
                mut local_timer: Local<Option<Timer>>) {
    let timer = local_timer.get_or_insert_with(|| Timer::from_seconds(3.0, false));
    if timer.tick(time.delta()).just_finished() {
        *local_timer = None;
        state.set(AppState::InGame).unwrap();
    }
}

//
// InGame state systems
//

fn move_system(mut moving_query: Query<(&Moving, &mut Transform)>) {
    for (moving, mut transform) in moving_query.iter_mut() {
        transform.translation += moving.velocity * 1.0 / 30.0;
    }
}

fn bounce(moving: &mut Moving, collision: &Collision) {
    match collision {
        Collision::Top => moving.velocity.y = moving.velocity.y.abs(),
        Collision::Bottom => moving.velocity.y = -moving.velocity.y.abs(),
        Collision::Left => moving.velocity.x = -moving.velocity.x.abs(),
        Collision::Right => moving.velocity.x = moving.velocity.x.abs(),
    }
}
fn ball_collide_system(mut ball_query: Query<(Entity, &Colliding, &mut Moving, &Transform), With<Ball>>,
                       colliding_query: Query<(Entity, &Colliding, &Transform)>,
                       mut events: EventWriter<PongEvent>) {


    for (ball_entity, ball_colliding, mut ball_moving, ball_transform) in ball_query.iter_mut() {
        for (other_entity, other_colliding, other_transform) in colliding_query.iter() {
            if ball_entity == other_entity {
                continue;
            }
            let collision = collide(ball_transform.translation, ball_colliding.size,
                                    other_transform.translation, other_colliding.size);
            if let Some(c) = collision {
                match other_colliding.kind {
                    Collider::Wall | Collider::Ball | Collider::Paddle(_) => bounce(&mut ball_moving, &c),
                    Collider::Goal(player) => events.send(PongEvent::Goal(player)),
                }
            }
        }
    }
}

fn event_listener_system(mut events: EventReader<PongEvent>,
                         mut score: ResMut<Score>,
                         mut app_state: ResMut<State<AppState>>) {
    for e in events.iter() {
        match e {
            PongEvent::Goal(player) => {
                match player {
                    Player::Left => {
                        score.left += 1;
                    },
                    Player::Right => {
                        score.right += 1;
                    }
                };
                app_state.set(AppState::Goal).unwrap();
            },
        }
    }
}

fn goal_enter(mut commands: Commands,
              asset_server: Res<AssetServer>,
              score: Res<Score>,
              mut score_text_query: Query<(&ScoreText, &mut Text)>) {
    // println!("Goal {} - {}", score.left, score.right);
    for (ScoreText(player), mut text) in score_text_query.iter_mut() {
        match player {
            Player::Left => text.sections[0].value = format!("{}", score.left),
            Player::Right => text.sections[0].value = format!("{}", score.right),
        }
    }

    commands.spawn_bundle(TextBundle {
        text: Text::with_section("G O A L !", TextStyle {
            font: asset_server.load("fonts/DejaVuSansMono-Bold.ttf"),
            font_size: 100.0,
            color: Color::WHITE
        }, TextAlignment::default()),
        style: Style {
            margin: Rect::all(Val::Auto),
            align_self: AlignSelf::Center,
            ..Style::default()
        },
        ..Default::default()
    })
    .insert(GoalText);
}

fn goal_update(mut goal_text_query: Query<&mut Transform, With<GoalText>>,
               mut state: ResMut<State<AppState>>,
               score: Res<Score>,
               time: Res<Time>,
               mut local_timer: Local<Option<Timer>>) {
    let timer = local_timer.get_or_insert_with(|| Timer::from_seconds(3.0, false));
    if timer.tick(time.delta()).just_finished() {
        *local_timer = None;
        if score.left >= GOALS_TO_WIN || score.right >= GOALS_TO_WIN {
            state.set(AppState::Win).unwrap();
        } else {
            state.set(AppState::Ready).unwrap();
        }
    } else {
        for mut text_transform in goal_text_query.iter_mut() {
            text_transform.scale += Vec3::splat(1.0 * time.delta().as_secs_f32());
        }
    }
}

pub fn goal_exit(mut commands: Commands, goal_text_query: Query<Entity, With<GoalText>>) {
    for e in goal_text_query.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn win_enter(mut commands: Commands,
             asset_server: Res<AssetServer>,
             score: Res<Score>) {
    // println!("Win");
    let winner_text = if score.left > score.right { "Left" } else { "Right" };
    commands.spawn_bundle(TextBundle {
        text: Text::with_section(format!("{} player wins!", winner_text), TextStyle {
            font: asset_server.load("fonts/DejaVuSansMono-Bold.ttf"),
            font_size: 100.0,
            color: Color::WHITE
        }, TextAlignment::default()),
        style: Style {
            margin: Rect::all(Val::Auto),
            align_self: AlignSelf::Center,
            ..Style::default()
        },
        ..Default::default()
    })
    .insert(WinText);
}

fn win_update(mut win_text_query: Query<&mut Visibility, With<WinText>>,
               mut state: ResMut<State<AppState>>,
               time: Res<Time>,
               mut local_timer: Local<Option<Timer>>) {
    let timer = local_timer.get_or_insert_with(|| Timer::from_seconds(3.0, false));
    if timer.tick(time.delta()).just_finished() {
        *local_timer = None;
        state.set(AppState::Title).unwrap();
    } else {
        let blink_interval: f32 = 0.3;
        for mut text_visibility in win_text_query.iter_mut() {
            text_visibility.is_visible = timer.elapsed_secs().rem_euclid(blink_interval) > blink_interval / 2.0;
        }
    }
}

pub fn win_exit(mut commands: Commands, win_text_query: Query<Entity, With<WinText>>) {
    for e in win_text_query.iter() {
        commands.entity(e).despawn_recursive();
    }
}
