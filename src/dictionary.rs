use serde::{Deserialize, Serialize};
use serenity::prelude::{RwLock, TypeMapKey};
use std::{collections::HashMap, sync::Arc};

type Seed = String;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Dictionary {
    pub word_map: HashMap<Seed, String>,
}

pub struct DictionaryData;

impl TypeMapKey for DictionaryData {
    type Value = Arc<RwLock<Dictionary>>;
}
