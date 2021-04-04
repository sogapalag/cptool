use super::module::{Lib, Module};
use super::regexes::*;

#[allow(unused_imports)]
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::{read, write, File},
    io::{prelude::*, BufReader},
    path::{Path, PathBuf},
    str::from_utf8,
};

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
            if cfg!(feature = "no_comment") && is_comment(&l) {
                continue;
            }
            if cfg!(feature = "no_doc_comment") && is_doc(&l) {
                continue;
            }

            let l = l.replace("crate::", "crate::cplib::"); // TODO avoid hard coded.
            out += &l;
            out += "\n";
        }
    }
    Ok(out)
}

/// input: leaf-modules,
/// output: crate_name { sub_mod {} ... mod{..} }
pub struct Tool {
    n: usize,
    g: Vec<Vec<usize>>,
    tour: Vec<Module>,
}
impl Tool {
    pub fn new(lib: Lib) -> Self {
        let Lib { tour } = lib;
        let n = tour.len();
        let mut g = vec![vec![]; n];
        for v in 1..n {
            if let Some(u) = tour[v].parent {
                g[u].push(v);
            }
        }

        Self { n, g, tour }
    }
}
impl Tool {
    fn submodule_to_id(&self, p: usize, x: &str) -> Result<usize, Ber> {
        for &v in &self.g[p] {
            if self.tour[v].name == x {
                return Ok(v);
            }
        }
        Err(format!("submodule {} of {} not exist!", x, self.tour[p].name).into())
    }
    // Warning: always return first, for duplicate module name.
    fn module_to_id(&self, x: &str) -> Result<usize, Ber> {
        for v in 0..self.n {
            if self.tour[v].name == x {
                return Ok(v);
            }
        }
        Err(format!("module {} not exist!", x).into())
    }

    fn dfs(&self, u: usize, out: &mut String, marked: &[bool]) {
        if !marked[u] {
            return;
        }
        if self.g[u].is_empty() {
            *out += &get_content_ignore_test(&self.tour[u].path).expect("get content fail");
        } else {
            let mut buf = BufReader::new(File::open(&self.tour[u].path).unwrap());
            // deal with #[attr]
            for &v in &self.g[u] {
                let mut tmp = String::new();
                loop {
                    let mut l = String::new();
                    buf.read_line(&mut l).unwrap();
                    if let Some(name) = is_module_def(&l) {
                        assert_eq!(name, &self.tour[v].name);
                        if marked[v] {
                            tmp += get_module_def(&l).unwrap();
                            tmp += " {\n";
                            *out += &tmp;
                            self.dfs(v, out, marked);
                            *out += "}\n";
                        }
                        break;
                    } else {
                        if cfg!(feature = "no_comment") && is_comment(&l) {
                            continue;
                        }
                        if cfg!(feature = "no_doc_comment") && is_doc(&l) {
                            continue;
                        }
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
                        let v: Vec<&str> = mods.split("::").collect();
                        //let mut m = v.pop().unwrap();
                        //if self.module_to_id(m).is_err() {
                        //    m = v.pop().unwrap();
                        //}
                        //let m = self.module_to_id(m).unwrap();
                        // ONLY ALLOW format
                        // pub use self::sub::Struct;
                        let m = v[1];
                        let m = self.submodule_to_id(u, m).unwrap();
                        if !marked[m] {
                            continue;
                        }
                    }
                    *out += &l;
                    *out += "\n";
                }
            }
        }
    }

    fn mark_des(&self, u: usize, marked: &mut [bool]) {
        marked[u] = true;
        for &v in &self.g[u] {
            self.mark_des(v, marked);
        }
    }
    pub fn run(&self, mods: &[usize]) -> String {
        let mut marked = vec![false; self.n];

        let mut mark_anc = |v: usize| {
            marked[v] = true;
            let mut v = v;
            while let Some(u) = self.tour[v].parent {
                marked[u] = true;
                v = u;
            }
        };
        for &v in mods {
            mark_anc(v);
        }
        for &v in mods {
            self.mark_des(v, &mut marked);
        }

        let mut out = String::new();
        out += "pub mod ";
        out += &self.tour[0].name;
        out += "{\n";
        self.dfs(0, &mut out, &marked);
        out += "}\n";
        out
    }

    pub fn generate<S: AsRef<str>>(&self, mods: &[S]) -> String {
        let mods: Vec<usize> = mods
            .iter()
            .map(|m| self.module_to_id(m.as_ref()).unwrap())
            .collect();
        self.run(&mods)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use home::home_dir;

    #[test]
    fn view_tree() {
        let lib_toml = home_dir().unwrap().join("cptool/lib.toml");
        let lib = toml::from_slice(&read(lib_toml).unwrap()).unwrap();
        let t = Tool::new(lib);
        dbg!(&t.g);
    }
    #[test]
    fn single() {}
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
