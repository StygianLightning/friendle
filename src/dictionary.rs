use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type Seed = String;

#[derive(Serialize, Deserialize)]
struct Dictionary {
    word_map: HashMap<Seed, String>,
}
