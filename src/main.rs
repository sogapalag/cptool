use cptool::tool::Tool;
use cptool::workspace::Workspace;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::{read, read_to_string, write};
use std::io::BufReader;
use std::str::from_utf8;

struct Config {
    src: String,
    modules: Vec<String>,
}
impl Config {
    fn new(args: &[String]) -> Config {
        let mut src = args[1].clone();
        if !src.ends_with(".rs") {
            src += ".rs";
        }
        let modules = args[2..].to_vec();

        Config { src, modules }
    }
}

fn expand(hm: &HashMap<String, Vec<String>>, v: &mut Vec<String>, key: &str) {
    if hm.contains_key(key) {
        for val in &hm[key] {
            expand(hm, v, val);
        }
    } else {
        v.push(key.to_string());
    }
}

fn get_mods(hm: &HashMap<String, Vec<String>>, cs: &[String]) -> Vec<String> {
    let mut v = Vec::new();
    for c in cs {
        expand(hm, &mut v, c);
    }
    expand(hm, &mut v, "default");
    v.sort();
    v.dedup();
    v
}

/// write into .buffer.rs
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args);

    let root = env::current_dir()?;

    let lib = toml::from_slice(&read(root.join("lib.toml"))?)?;
    let workspace: Workspace = toml::from_slice(&read(root.join("workspace.toml"))?)?;

    let t = Tool::new(lib);
    let mods = get_mods(&workspace.components, &config.modules);
    let mut s = t.generate(&mods);

    let src = workspace.path.join(&config.src);
    //dbg!(&src);
    s += from_utf8(&read(src).expect("no src file"))?;

    write(root.join(".buffer.rs"), s)?;
    Ok(())
}
