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
            .add_system_set(SystemSet::on_exit(AppState::Ready).with_system(ready_exit))
            .add_system_set(SystemSet::on_update(AppState::InGame)
                .with_system(paddle_input)
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
    debug!("NewGame");
    score.left = 0;
    score.right = 0;
    for mut text in score_text_query.iter_mut() {
        text.sections[0].value = "0".to_string();
    }
    state.set(AppState::Ready).unwrap()
}

fn ready_enter(mut ball_query: Query<&mut Transform, With<Ball>>,
               mut ready_text_query: Query<(&mut Text, &mut Transform, &mut Style), (With<ReadyText>, Without<Ball>)>) {
    debug!("Ready");
    for mut transform in ball_query.iter_mut() {
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;
    }
    for (mut text, mut transform, mut style) in ready_text_query.iter_mut() {
        text.sections[0].value = format!("{}", READY_DURATION.ceil() as i32);
        transform.scale = Vec3::ONE;
        style.display = Display::Flex;
    }
}

fn ready_update(mut state: ResMut<State<AppState>>,
                time: Res<Time>,
                mut local_timer: Local<Option<Timer>>,
                mut ready_text_query: Query<(&mut Text, &mut Transform), With<ReadyText>>) {
    let timer = local_timer.get_or_insert_with(|| Timer::from_seconds(READY_DURATION, false));
    if timer.tick(time.delta()).just_finished() {
        *local_timer = None;
        state.set(AppState::InGame).unwrap();
    } else {
        for (mut text, mut text_transform) in ready_text_query.iter_mut() {
            let current_time = timer.elapsed_secs();
            let previous_time = timer.elapsed_secs() - time.delta().as_secs_f32();
            if current_time.floor() > previous_time.floor() {
                text.sections[0].value = format!("{}", (READY_DURATION - current_time).ceil() as i32);
                text_transform.scale = Vec3::ONE;
            } else {
                text_transform.scale += Vec3::splat(1.0 * time.delta().as_secs_f32());
            }
        }
    }
}

pub fn ready_exit(mut ready_text_query: Query<&mut Style, With<ReadyText>>) {
    for mut style in ready_text_query.iter_mut() {
        style.display = Display::None;
    }
}
//
// InGame state systems
//

fn paddle_input(mut paddle_query: Query<(&Paddle, &mut Moving)>,
                keyboard_input: Res<Input<KeyCode>>) {
    let mut left_velocity = Vec3::ZERO;
    let mut right_velocity = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::A) {
        left_velocity.y += PADDLE_SPEED;
    }
    if keyboard_input.pressed(KeyCode::Z) {
        left_velocity.y -= PADDLE_SPEED;
    }
    if keyboard_input.pressed(KeyCode::K) {
        right_velocity.y += PADDLE_SPEED;
    }
    if keyboard_input.pressed(KeyCode::M) {
        right_velocity.y -= PADDLE_SPEED;
    }
    
    for (Paddle(player), mut paddle_moving) in paddle_query.iter_mut() {
        paddle_moving.velocity = match player {
            Player::Left => left_velocity,
            Player::Right => right_velocity
        };
    }
}

fn move_system(mut moving_query: Query<(&Moving, &mut Transform)>, time: Res<Time>) {
    for (moving, mut transform) in moving_query.iter_mut() {
        transform.translation += moving.velocity * time.delta().as_secs_f32();
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
                    Collider::Wall | Collider::Ball | Collider::Paddle => bounce(&mut ball_moving, &c),
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

fn goal_enter(mut goal_text_query: Query<(&mut Transform, &mut Style), (With<GoalText>, Without<ScoreText>)>,
              score: Res<Score>,
              mut score_text_query: Query<(&ScoreText, &mut Text)>) {
    debug!("Goal {} - {}", score.left, score.right);
    for (ScoreText(player), mut text) in score_text_query.iter_mut() {
        match player {
            Player::Left => text.sections[0].value = format!("{}", score.left),
            Player::Right => text.sections[0].value = format!("{}", score.right),
        }
    }
    for (mut transform, mut style) in goal_text_query.iter_mut() {
        transform.scale = Vec3::ONE;
        style.display = Display::Flex;
    }
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

pub fn goal_exit(mut goal_text_query: Query<&mut Style, With<GoalText>>) {
    for mut style in goal_text_query.iter_mut() {
        style.display = Display::None;
    }
}

fn win_enter(mut commands: Commands,
             asset_server: Res<AssetServer>,
             score: Res<Score>) {
    debug!("Win");
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
