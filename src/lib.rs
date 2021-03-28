use regex::Regex;
#[allow(unused_imports)]
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::{read, write, File},
    io::{prelude::*, BufReader},
    path::{Path, PathBuf},
    str::from_utf8,
};
#[allow(unused_imports)]
use walkdir::WalkDir;

type HM = HashMap<String, Vec<String>>;
type Ber = Box<dyn Error + 'static>;

// ignore tail tests
fn get_content_ignore_test(f: &Path) -> Result<String, std::io::Error> {
    let f = BufReader::new(File::open(f)?);
    let mut out = String::new();
    for l in f.lines() {
        if let Ok(l) = l {
            if l.starts_with("#[cfg(test)]") || l.starts_with("#[test]") {
                break;
            }
            let l = l.replace("crate::", "crate::cplib::"); // TODO avoid hard coded.
            out += &l;
            out += "\n";
        }
    }
    Ok(out)
}
fn is_module_def(text: &str) -> Option<&str> {
    let RE = Regex::new(r"^[[:space:]]*(pub )?mod (?P<name>[[:word:]]+);").unwrap();
    RE.captures(text)
        .and_then(|cap| cap.name("name").map(|name| name.as_str()))
}

fn get_module_def(text: &str) -> Option<&str> {
    let RE = Regex::new(r"^(?P<def>(pub )?mod ([[:word:]]+));").unwrap();
    RE.captures(text)
        .and_then(|cap| cap.name("def").map(|x| x.as_str()))
}

fn get_use(text: &str) -> Option<&str> {
    let RE = Regex::new(r"use (?P<use>[[:word:]]+(::[[[:word:]]\*]+)*);").unwrap();
    RE.captures(text)
        .and_then(|cap| cap.name("use").map(|x| x.as_str()))
}
fn get_sub_modules(text: &str) -> Vec<&str> {
    text.lines().filter_map(|l| is_module_def(l)).collect()
}

/// input: leaf-modules,
/// output: crate_name { sub_mod {} ... mod{..} }
pub struct Tool {
    crate_name: &'static str,
    n: usize,
    tour: Vec<PathBuf>,
    modules: Vec<String>,
    def_path: Vec<PathBuf>,
    g: Vec<Vec<usize>>,
    pa: Vec<usize>,
}
impl Tool {
    pub fn new(crate_name: &'static str) -> Self {
        let mut tour = Vec::<PathBuf>::new();

        let home = home::home_dir();
        let root = home.unwrap().join(crate_name).join("src");
        Self::get_tour(&root, &mut tour).expect("fail tour");

        let n = tour.len();
        let id = |x| {
            for i in 0..n {
                if tour[i] == x {
                    return i;
                }
            }
            unreachable!()
        };
        // construct other data
        let mut g = vec![vec![]; n];
        let mut pa = vec![0; n];
        for v in 1..n {
            let u = id(tour[v].parent().unwrap());
            g[u].push(v);
            pa[v] = u;
        }
        let mut modules: Vec<String> = vec![];
        let mut def_path: Vec<PathBuf> = vec![];
        for v in 0..n {
            let x = &tour[v];
            let name = x.file_stem().unwrap().to_str().unwrap();
            def_path.push(if x.is_file() {
                x.clone()
            } else {
                x.join(if name == "src" { "lib.rs" } else { "mod.rs" })
            });
            modules.push(if name == "src" { crate_name } else { name }.to_string());
        }

        Self {
            crate_name,
            n,
            tour,
            modules,
            def_path,
            g,
            pa,
        }
    }
}
impl Tool {
    fn path_to_id(&self, x: &Path) -> Result<usize, Ber> {
        for i in 0..self.n {
            if self.tour[i] == x {
                return Ok(i);
            }
        }
        Err(format!("path {} not exist!", x.to_str().unwrap()).into())
    }
    fn module_to_id(&self, x: &str) -> Result<usize, Ber> {
        for i in 0..self.n {
            if self.modules[i] == x {
                return Ok(i);
            }
        }
        Err(format!("module {} not exist!", x).into())
    }

    /// get dfs tour
    fn get_tour(u: &Path, tour: &mut Vec<PathBuf>) -> Result<(), Ber> {
        tour.push(u.to_path_buf());

        if u.is_file() {
            return Ok(());
        }

        let name = u.file_stem().unwrap().to_str().unwrap();
        let file = u.join(if name == "src" { "lib.rs" } else { "mod.rs" });

        let file = BufReader::new(File::open(file)?);
        let mut subs: Vec<String> = vec![];
        file.lines().filter_map(|l| l.ok()).for_each(|l| {
            if let Some(l) = is_module_def(&l) {
                subs.push(l.to_string());
            }
        });

        // visit sub mods defined in lib.rs/mod.rs
        for s in subs {
            let s2 = s.to_string() + ".rs";
            let v = u.join(&s);
            let v2 = u.join(&s2);
            assert!(v.exists() ^ v2.exists());
            let v = if v.exists() { v } else { v2 };
            Self::get_tour(&v, tour);
        }
        Ok(())
    }
    fn dfs(&self, u: usize, out: &mut String, marked: &[bool]) {
        if !marked[u] {
            return;
        }
        let name = &self.modules[u];
        if self.g[u].is_empty() {
            *out += &get_content_ignore_test(&self.def_path[u]).expect("get content fail");
        } else {
            let mut buf = BufReader::new(File::open(&self.def_path[u]).unwrap());
            // deal with #[attr]
            for &v in &self.g[u] {
                let mut tmp = String::new();
                loop {
                    let mut l = String::new();
                    buf.read_line(&mut l);
                    if let Some(name) = is_module_def(&l) {
                        assert_eq!(name, &self.modules[v]);
                        if marked[v] {
                            tmp += get_module_def(&l).unwrap();
                            tmp += " {\n";
                            *out += &tmp;
                            self.dfs(v, out, marked);
                            *out += "}\n";
                        }
                        break;
                    } else {
                        // #[attr]
                        tmp += &l;
                    }
                }
            }
            // deal with re-export
            for l in buf.lines() {
                if let Ok(l) = l {
                    // check tail mod marked?
                    if let Some(mods) = get_use(&l) {
                        let mut v: Vec<&str> = mods.split("::").collect();
                        let mut m = v.pop().unwrap();
                        if self.module_to_id(m).is_err() {
                            m = v.pop().unwrap();
                        }
                        let m = self.module_to_id(m).unwrap();
                        if !marked[m] {
                            break;
                        }
                    }
                    *out += &l;
                }
            }
        }
    }

    fn run(&self, mods: &[usize]) -> String {
        let mut marked = vec![false; self.n];

        let mut mark_anc = |mut v| loop {
            marked[v] = true;
            if v == 0 {
                break;
            }
            v = self.pa[v];
        };
        for &v in mods {
            mark_anc(v);
        }

        let mut out = String::new();
        out += "pub mod ";
        out += self.crate_name;
        out += "{\n";
        self.dfs(0, &mut out, &marked);
        out += "}\n";
        out
    }

    fn run_from_module<S: AsRef<str>>(&self, mods: &[S]) -> String {
        let mods: Vec<usize> = mods
            .iter()
            .map(|m| self.module_to_id(m.as_ref()).unwrap())
            .collect();
        self.run(&mods)
    }
    pub fn write<P: AsRef<Path>, S: AsRef<str>>(&mut self, out: P, files: &[S]) {
        write(out, self.run_from_module(files)).expect("write fail");
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use home::home_dir;
    use walkdir::WalkDir;

    #[test]
    fn view_tree() {
        let t = Tool::new("cplib");
        dbg!(t.tour);
        dbg!(t.g);
    }
    #[test]
    fn single() {
        let mut t = Tool::new("cplib");
        let tmp = "dsu";
        let out = t.run_from_module(&[tmp]);
        dbg!(out);
    }
    //#[test]
    //fn two() {
    //    let mut t = Tool::new("cplib");
    //    let tmp = "dsu";
    //    let tmp2 = "algebra";
    //    let out = t.run_from_module(&[tmp, tmp2]);
    //    dbg!(out);
    //}
    //#[test]
    //fn file_not_exist() {
    //    let t = Tool::new("cplib");
    //    let tmp = "nothing";
    //    if let Err(e) = t.into_path(tmp) {
    //        assert_eq!(e.to_string(), format!("{}.rs not exist!", tmp));
    //    }
    //    let tmp = "ds";
    //    if let Err(e) = t.into_path(tmp) {
    //        assert_eq!(e.to_string(), format!("{}.rs not exist!", tmp));
    //    }
    //}
    //#[test]
    //fn test_ignore() {
    //    let mut t = Tool::new("cplib");
    //    let tmp = "cmp";
    //    let out = t.run_from_module(&[tmp]);
    //    dbg!(out);
    //}
    //#[test]
    //fn write() {
    //    let p = "write.rs";
    //    let mut t = Tool::new("cplib");
    //    let tmp = "dsu";
    //    t.write(p, &[tmp]);
    //}
}
