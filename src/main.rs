use cptool::Tool;
use std::env;

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
fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args);

    let cplib = "cplib"; // "~/cplib"
    let mut t = Tool::new(cplib);
    t.write(config.out, &config.modules);
}
