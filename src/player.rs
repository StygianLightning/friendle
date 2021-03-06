use serenity::prelude::TypeMapKey;
use std::sync::Mutex;

use std::{collections::HashMap, sync::Arc};

use crate::model::game::Game;

#[derive(Debug, Default)]
pub struct PlayerState {
    pub games_per_player: HashMap<u64, Game>,
}

impl TypeMapKey for PlayerState {
    type Value = Arc<Mutex<PlayerState>>;
}
