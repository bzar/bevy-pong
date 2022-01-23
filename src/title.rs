use bevy::{
    prelude::*,
    app::AppExit
};
use crate::types::*;

pub struct TitlePlugin;

impl Plugin for TitlePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(SystemSet::on_enter(AppState::Title).with_system(title_enter))
            .add_system_set(SystemSet::on_update(AppState::Title).with_system(title_update))
            .add_system_set(SystemSet::on_exit(AppState::Title).with_system(title_exit));
    }
}

pub fn title_enter(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            flex_direction: FlexDirection::ColumnReverse,
            align_items: AlignItems::Center,
            align_content: AlignContent::Center,
            ..Default::default()
        },
        color: Color::NONE.into(),
        ..Default::default()
    })
    .insert(TitleText)
    .with_children(|parent| {
        parent.spawn_bundle(TextBundle {
            text: Text::with_section("Pong", TextStyle {
                font: asset_server.load("fonts/DejaVuSansMono-Bold.ttf"),
                font_size: 100.0,
                color: Color::WHITE
            }, TextAlignment::default()),
            style: Style {
                ..Default::default()
            },
            ..Default::default()
        });

        parent.spawn_bundle(TextBundle {
            text: Text::with_section("Press SPACE to start!", TextStyle {
                font: asset_server.load("fonts/DejaVuSansMono-Bold.ttf"),
                font_size: 70.0,
                color: Color::GRAY
            }, TextAlignment::default()),
            style: Style {
                ..Default::default()
            },
            ..Default::default()
        });
    });
}

pub fn title_update(mut exit_events: EventWriter<AppExit>,
                    mut state: ResMut<State<AppState>>,
                    keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_released(KeyCode::Space) {
        state.set(AppState::NewGame).unwrap()
    } else if keyboard_input.just_released(KeyCode::Escape) {
        exit_events.send(AppExit);
    }
}

pub fn title_exit(mut commands: Commands, title_text_query: Query<Entity, With<TitleText>>) {
    for e in title_text_query.iter() {
        commands.entity(e).despawn_recursive();
    }
}

