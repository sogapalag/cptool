use cptool::tool::Tool;
use std::env;
use std::error::Error;
use std::fs::{read, write};

struct Config {
    out: String,
    modules: Vec<String>,
}
impl Config {
    fn new(args: &[String]) -> Config {
        //let out = args[1].clone();
        let out = String::from(".buffer.rs");
        //dbg!(&out);
        let modules = args[1..].to_vec();

        Config { out, modules }
    }
}

/// write into .buffer.rs
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args);

    let root = env::current_dir()?;
    //dbg!(root);
    let lib = toml::from_slice(&read(root.join("lib.toml"))?)?;
    let mut t = Tool::new(lib);
    write(config.out, t.generate(&config.modules))?;
    Ok(())
}
