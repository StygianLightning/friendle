use serde::{Deserialize, Serialize};
use serenity::prelude::TypeMapKey;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WordList {
    pub words: HashSet<String>,
}

impl TypeMapKey for WordList {
    type Value = Arc<WordList>;
}
