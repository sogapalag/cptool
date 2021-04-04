use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Workspace {
    pub path: PathBuf,
    pub components: HashMap<String, Vec<String>>,
}
