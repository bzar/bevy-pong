use bevy::prelude::*;
use crate::types::*;
use crate::consts::*;


pub fn setup(mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<StandardMaterial>>,
         asset_server: Res<AssetServer>) {
    debug!("Setup");
    // Floor
    let floor_material = materials.add(StandardMaterial {
        base_color: Color::rgb(1.0, 1.0, 1.0),
        metallic: 0.0,
        reflectance: 0.0,
        ..StandardMaterial::default()
    });
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(AREA_WIDTH, AREA_HEIGHT, WALL_THICKNESS))),
        material: floor_material,
        transform: Transform::from_xyz(0.0, 0.0, -WALL_THICKNESS),
        ..Default::default()
    });

    // Walls
    {
        let material = materials.add(Color::rgb(0.5, 0.5, 0.1).into());
        let size = Vec3::new(AREA_WIDTH, WALL_THICKNESS, WALL_THICKNESS);
        let mesh = meshes.add(Mesh::from(shape::Box::new(size.x, size.y, size.z)));
        let data = [AREA_HEIGHT/2.0, -AREA_HEIGHT/2.0];
        for y in data {
            commands.spawn_bundle(PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                transform: Transform::from_xyz(0.0, y, 0.0),
                ..Default::default()
            }).insert(Colliding { kind: Collider::Wall, size: size.truncate() });
        }
    }

    // Goals
    {
        let size = Vec3::new(WALL_THICKNESS, AREA_HEIGHT - WALL_THICKNESS, WALL_THICKNESS);
        let mesh = meshes.add(Mesh::from(shape::Box::new(size.x, size.y, size.z)));
        let data = [
            (Player::Right, Color::rgb(0.8, 0.1, 0.1), (WALL_THICKNESS - AREA_WIDTH) / 2.0),
            (Player::Left, Color::rgb(0.1, 0.1, 0.8), (AREA_WIDTH - WALL_THICKNESS) / 2.0)
        ];
        for (player, color, x) in data {
            commands.spawn_bundle(PbrBundle {
                mesh: mesh.clone(),
                material: materials.add(color.into()),
                transform: Transform::from_xyz(x, 0.0, 0.0),
                ..Default::default()
            }).insert(Colliding { kind: Collider::Goal(player), size: size.truncate() });
        }
    }

    // Ball
    let ball_material = StandardMaterial {
        base_color: Color::rgb(1.0, 1.0, 1.0),
        emissive: Color::rgb(1.0, 1.0, 1.0),
        reflectance: 0.0,
        ..StandardMaterial::default()
    };
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Icosphere { radius: BALL_SIZE/2.0, subdivisions: 64 })),
        material: materials.add(ball_material),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    })
    .insert(Ball)
    .insert(Moving { velocity: Vec3::new(BALL_SPEED, BALL_SPEED, 0.0) })
    .insert(Colliding { kind: Collider::Ball, size: Vec2::new(BALL_SIZE, BALL_SIZE) })
    .with_children(|parent| {
        parent.spawn_bundle(PointLightBundle {
            point_light: PointLight {
                intensity: 50.0,
                shadows_enabled: true,
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, 0.0, BALL_SIZE * 2.0),
            ..Default::default()
        });
    });

    // Paddles
    {
        let size = Vec3::new(PADDLE_THICKNESS, PADDLE_LENGTH, WALL_THICKNESS);
        let mesh = meshes.add(Mesh::from(shape::Box::new(size.x, size.y, size.z)));
        let data = [
            (Player::Left, -AREA_WIDTH/2.0 + WALL_THICKNESS + PADDLE_THICKNESS, Color::rgb(0.6, 0.3, 0.3)),
            (Player::Right, AREA_WIDTH/2.0 - WALL_THICKNESS - PADDLE_THICKNESS, Color::rgb(0.3, 0.3, 0.6)),
        ];
        for (player, x, color) in data {
            let transform = Transform::from_xyz(x, 0.0, 0.0);
            let material = materials.add(color.into());
            commands.spawn_bundle(PbrBundle { mesh: mesh.clone(), material, transform, ..Default::default() })
                .insert(Colliding { kind: Collider::Paddle, size: size.truncate() })
                .insert(Moving::default())
                .insert(Paddle(player));
        }
    }

    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.0, -AREA_HEIGHT/2.0, AREA_WIDTH).looking_at(Vec3::ZERO, Vec3::Z),
        ..Default::default()
    });

    // lights
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            color: Color::rgb(1.0, 0.3, 0.3),
            intensity: 1500.0,
            ..Default::default()
        },
        transform: Transform::from_xyz(-AREA_WIDTH/4.0, 0.0, 10.0),
        ..Default::default()
    });
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            color: Color::rgb(0.3, 0.3, 1.0),
            intensity: 1500.0,
            ..Default::default()
        },
        transform: Transform::from_xyz(AREA_WIDTH/4.0, 0.0, 10.0),
        ..Default::default()
    });

    // UI camera
    commands.spawn_bundle(UiCameraBundle::default());

    let font = asset_server.load("fonts/DejaVuSansMono-Bold.ttf");
    // Scores
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 50.0,
        color: Color::WHITE
    };

    commands.spawn_bundle(TextBundle {
        text: Text::with_section("0", text_style.clone(), TextAlignment::default()),
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(ScoreText(Player::Left));

    commands.spawn_bundle(TextBundle {
        text: Text::with_section("0", text_style, TextAlignment::default()),
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(5.0),
                right: Val::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(ScoreText(Player::Right));

    commands.spawn_bundle(TextBundle {
        text: Text::with_section("", TextStyle {
            font: font.clone(),
            font_size: 100.0,
            color: Color::WHITE
        }, TextAlignment::default()),
        style: Style {
            display: Display::None,
            margin: Rect::all(Val::Auto),
            align_self: AlignSelf::Center,
            ..Style::default()
        },
        ..Default::default()
    })
    .insert(ReadyText);

    commands.spawn_bundle(TextBundle {
        text: Text::with_section("G O A L !", TextStyle {
            font: font.clone(),
            font_size: 100.0,
            color: Color::WHITE
        }, TextAlignment::default()),
        style: Style {
            display: Display::None,
            margin: Rect::all(Val::Auto),
            align_self: AlignSelf::Center,
            ..Style::default()
        },
        ..Default::default()
    })
    .insert(GoalText);
}
