use cptool::Tool;
use std::env;

struct Config<'a> {
    out: String,
    modules: Vec<&'a str>,
}
impl Config<'_> {
    fn new(args: &[String]) -> Config {
        //let out = args[1].clone();
        let mut out = env!("CARGO").clone().to_string();
        out += ".buffer.rs";
        let modules = args[1..].iter().map(|s| s.as_str()).collect();

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
