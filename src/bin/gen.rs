use cptool::module::{Lib, Module};
use cptool::regexes::is_module_def;
use home::home_dir;
use std::env;
use std::fs::{write, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

type Ber = Box<dyn std::error::Error>;

/// get dfs tour
fn get_tour(u: &Path, tour: &mut Vec<Module>, pa: Option<usize>) -> Result<(), Ber> {
    let name = if pa.is_some() {
        u.file_stem().unwrap().to_str().unwrap()
    } else {
        u.parent().unwrap().file_stem().unwrap().to_str().unwrap()
    };
    let name = String::from(name);
    let path = if u.is_file() {
        u.to_path_buf()
    } else {
        if pa.is_some() {
            u.join("mod.rs")
        } else {
            u.join("lib.rs")
        }
    };

    let uu = Module {
        id: tour.len(),
        parent: pa,
        name,
        path,
    };
    tour.push(uu.clone());

    if u.is_file() {
        return Ok(());
    }

    // filter sub mod in path
    let file = BufReader::new(File::open(uu.path)?);
    let mut subs: Vec<String> = vec![];
    file.lines().filter_map(|l| l.ok()).for_each(|l| {
        if let Some(l) = is_module_def(&l) {
            if l != "tests" {
                subs.push(l.to_string())
            };
        }
    });

    // visit sub mods defined in lib.rs/mod.rs
    for s in subs {
        let s2 = s.to_string() + ".rs";
        let v = u.join(&s);
        let x = v.join("mod.rs");
        let y = u.join(&s2);
        assert!(x.exists() ^ y.exists(), "x={:?}, y={:?}", x, y);
        let v = if x.exists() { v } else { y };
        get_tour(&v, tour, Some(uu.id));
    }
    Ok(())
}

fn main() -> Result<(), Ber> {
    let args: Vec<String> = env::args().collect();
    let lib = args[1].clone();
    let path = home_dir().unwrap().join(lib).join("src");
    let mut tour = Vec::new();
    get_tour(&path, &mut tour, None);

    let lib = Lib { tour };

    let toml = toml::to_string(&lib)?;
    let out = env::current_dir()?.join("lib.toml");
    write(out, toml);
    Ok(())
}
