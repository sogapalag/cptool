use cptool::workspace::Workspace;
use std::collections::HashMap;
use std::path::PathBuf;

fn main() {
    let mut w = Workspace {
        path: PathBuf::from("/home/peng/cplib/src/lib.rs"),

        components: HashMap::new(),
    };
    w.components.insert("s".into(), vec![]);
    let s = toml::to_string(&w).unwrap();
    dbg!(s);
}
