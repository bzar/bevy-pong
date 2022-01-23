use bevy::prelude::*;

mod consts;
mod types;
mod setup;
mod title;
mod game;

use types::*;
use setup::setup;
use title::TitlePlugin;
use game::GamePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_event::<PongEvent>()
        .insert_resource(Score { left: 0, right: 0 })
        .add_startup_system(setup)
        .add_plugin(TitlePlugin)
        .add_plugin(GamePlugin)
        .add_state(AppState::Title)
        .run();
}
