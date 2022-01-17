use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision}
};

const AREA_WIDTH: f32 = 20.0;
const AREA_HEIGHT: f32 = 10.0;
const WALL_THICKNESS: f32 = 0.2;
const BALL_SIZE: f32 = 0.2;
const PADDLE_LENGTH: f32 = 1.0;
const PADDLE_THICKNESS: f32 = 0.3;

#[derive(Copy, Clone)]
enum Player { Left, Right }

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Paddle(Player);

#[derive(Component)]
struct Moving {
    velocity: Vec3
}

#[derive(Copy, Clone)]
enum Collider {
    Wall,
    Ball,
    Paddle(Player),
    Goal(Player)
}

#[derive(Component)]
struct Colliding {
    kind: Collider ,
    size: Vec2,
}


enum PongEvent {
    Goal(Player),
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_event::<PongEvent>()
        .add_startup_system(setup)
        .add_system(move_system)
        .add_system(ball_collide_system)
        .add_system(event_listener_system)
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    println!("Setup");
    // Floor
    let floor_material = materials.add(Color::rgb(0.3, 0.5, 0.3).into());
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(AREA_WIDTH, AREA_HEIGHT, WALL_THICKNESS))),
        material: floor_material,
        ..Default::default()
    });

    // Walls
    let wall_material = materials.add(Color::rgb(0.5, 0.5, 0.1).into());
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(AREA_WIDTH, WALL_THICKNESS, WALL_THICKNESS))),
        material: wall_material.clone(),
        transform: Transform::from_xyz(0.0, AREA_HEIGHT/2.0, WALL_THICKNESS),
        ..Default::default()
    })
    .insert(Colliding { kind: Collider::Wall, size: Vec2::new(AREA_WIDTH, WALL_THICKNESS) });
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(AREA_WIDTH, WALL_THICKNESS, WALL_THICKNESS))),
        material: wall_material,
        transform: Transform::from_xyz(0.0, -AREA_HEIGHT/2.0, WALL_THICKNESS),
        ..Default::default()
    })
    .insert(Colliding { kind: Collider::Wall, size: Vec2::new(AREA_WIDTH, WALL_THICKNESS) });

    // Goals
    let goal_size = Vec3::new(WALL_THICKNESS, AREA_HEIGHT - WALL_THICKNESS, WALL_THICKNESS);
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(goal_size.x, goal_size.y, goal_size.z))),
        material: materials.add(Color::rgb(0.8, 0.1, 0.1).into()),
        transform: Transform::from_xyz(-AREA_WIDTH/2.0, 0.0, goal_size.z),
        ..Default::default()
    })
    .insert(Colliding { kind: Collider::Goal(Player::Right), size: goal_size.truncate() });
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(goal_size.x, goal_size.y, goal_size.z))),
        material: materials.add(Color::rgb(0.1, 0.1, 0.8).into()),
        transform: Transform::from_xyz(AREA_WIDTH/2.0, 0.0, goal_size.z),
        ..Default::default()
    })
    .insert(Colliding { kind: Collider::Goal(Player::Left), size: goal_size.truncate() });

    // Ball
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: BALL_SIZE })),
        material: materials.add(Color::rgb(0.8, 0.1, 0.1).into()),
        transform: Transform::from_xyz(0.0, 0.0, BALL_SIZE),
        ..Default::default()
    })
    .insert(Ball)
    .insert(Moving { velocity: Vec3::new(2.0, 5.0, 0.0) })
    .insert(Colliding { kind: Collider::Ball, size: Vec2::new(BALL_SIZE, BALL_SIZE) });

    // Paddles
    let paddle_size = Vec3::new(PADDLE_THICKNESS, PADDLE_LENGTH, WALL_THICKNESS);
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(paddle_size.x, paddle_size.y, paddle_size.z))),
        material: materials.add(Color::rgb(0.6, 0.3, 0.3).into()),
        transform: Transform::from_xyz(-AREA_WIDTH/2.0 + WALL_THICKNESS + PADDLE_THICKNESS, 0.0, paddle_size.z),
        ..Default::default()
    })
    .insert(Colliding { kind: Collider::Paddle(Player::Left), size: paddle_size.truncate() });

    let paddle_size = Vec3::new(PADDLE_THICKNESS, PADDLE_LENGTH, WALL_THICKNESS);
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(paddle_size.x, paddle_size.y, paddle_size.z))),
        material: materials.add(Color::rgb(0.3, 0.3, 0.6).into()),
        transform: Transform::from_xyz(AREA_WIDTH/2.0 - WALL_THICKNESS - PADDLE_THICKNESS, 0.0, paddle_size.z),
        ..Default::default()
    })
    .insert(Colliding { kind: Collider::Paddle(Player::Right), size: paddle_size.truncate() });
    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 4.0, 8.0),
        ..Default::default()
    });
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.0, -AREA_HEIGHT/2.0, AREA_WIDTH).looking_at(Vec3::ZERO, Vec3::Z),
        ..Default::default()
    });
}

fn move_system(mut moving_query: Query<(&Moving, &mut Transform)>) {
    for (moving, mut transform) in moving_query.iter_mut() {
        transform.translation += moving.velocity * 1.0 / 60.0;
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
fn ball_collide_system(mut ball_query: Query<(&Ball, Entity, &Colliding, &mut Moving, &Transform)>,
                       colliding_query: Query<(Entity, &Colliding, &Transform)>,
                       mut events: EventWriter<PongEvent>) {


    for (_ball, ball_entity, ball_colliding, mut ball_moving, ball_transform) in ball_query.iter_mut() {
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

fn event_listener_system(mut events: EventReader<PongEvent>, mut ball_query: Query<(&Ball, &mut Transform)>) {
    for e in events.iter() {
        match e {
            PongEvent::Goal(player) => {
                for (_ball, mut transform) in ball_query.iter_mut() {
                    transform.translation.x = 0.0;
                    transform.translation.y = 0.0;
                }
                match player {
                    Player::Left => println!("Left made a goal!"),
                    Player::Right => println!("Right made a goal!")
                }
            },
        }
    }
}
