mod select;
mod gameplay;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>();

    app.add_plugins((
        select::plugin,
        gameplay::plugin,
    ));
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum Screen {
    #[default]
    Select,
    Gameplay,
    Loading,
}
