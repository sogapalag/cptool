#[allow(unused_imports)]
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::{read, write, File},
    io::{prelude::*, BufReader},
    path::{Path, PathBuf},
    result::Result,
    str::from_utf8,
};
#[allow(unused_imports)]
use walkdir::WalkDir;

type HM = HashMap<PathBuf, Vec<PathBuf>>;
type Ber = Box<dyn Error + 'static>;

fn get_content(f: &Path) -> Result<Vec<u8>, std::io::Error> {
    read(f)
}
// ignore tail tests
fn get_content_ignore_test(f: &Path) -> Result<Vec<u8>, std::io::Error> {
    let f = BufReader::new(File::open(f)?);
    let mut out = Vec::<u8>::new();
    for l in f.lines() {
        if let Ok(l) = l {
            if l.starts_with("#[cfg(test)]") || l.starts_with("#[test]") {
                break;
            }
            for &c in l.as_bytes() {
                out.push(c);
            }
        }
    }
    Ok(out)
}
fn build_tree(root: &Path) -> Result<HM, Ber> {
    let mut g = HashMap::<PathBuf, Vec<PathBuf>>::new();
    for f in WalkDir::new(root) {
        let f = f?.into_path();
        if f != root {
            let p = f.parent().unwrap();
            if let Some(vs) = g.get_mut(p) {
                vs.push(f);
            } else {
                g.insert(p.to_path_buf(), vec![f]);
            }
        }
    }
    Ok(g)
}

/// input: leaf-modules,
/// output: crate_name { sub_mod {} ... mod{..} }
pub struct Tool {
    g: HM,
    marked: HashSet<PathBuf>,
    root: PathBuf,
    crate_name: &'static str,
}
impl Tool {
    pub fn new(crate_name: &'static str) -> Self {
        let home = home::home_dir();
        let root = home.unwrap().join(crate_name).join("src");
        Self {
            g: build_tree(&root).unwrap(),
            marked: HashSet::<PathBuf>::new(),
            root: root.clone(),
            crate_name: crate_name.clone(),
        }
    }
    fn dfs(&self, out: &mut String, u: &Path) {
        let name = u.file_stem().unwrap().to_str().unwrap();
        let name = if name == "src" { self.crate_name } else { name };
        *out += "mod ";
        *out += name;
        *out += "{";
        if u.is_file() {
            //*out += std::str::from_utf8(&get_content(u).expect("file not exist")).unwrap();
            *out +=
                std::str::from_utf8(&get_content_ignore_test(u).expect("file not exist")).unwrap();
        } else {
            for v in &self.g[u] {
                if self.marked.contains(v) {
                    self.dfs(out, v);
                }
            }
        }
        *out += "}";
    }

    fn run_from_path(&mut self, files: &[PathBuf]) -> String {
        self.marked.clear();

        let mut mark_anc = |f: &Path| {
            for p in f.ancestors() {
                if p == self.root {
                    break;
                }
                self.marked.insert(p.to_path_buf());
            }
        };
        for f in files {
            mark_anc(f);
        }

        let mut out = String::new();
        self.dfs(&mut out, &self.root);
        out
    }

    fn into_path(&self, f: &str) -> Result<PathBuf, Ber> {
        for v in WalkDir::new(&self.root) {
            let v = v?.into_path();
            let name = v.file_stem().unwrap().to_str().unwrap();
            if f == name && v.is_file() {
                return Ok(v);
            }
        }
        Err(format!("{}.rs not exist!", f).into())
    }
    fn run_from_module(&mut self, files: &[&str]) -> String {
        let files: Vec<PathBuf> = files.iter().map(|&f| self.into_path(f).unwrap()).collect();
        self.run_from_path(&files)
    }
    pub fn write<P: AsRef<Path>>(&mut self, out: P, files: &[&str]) {
        write(out, self.run_from_module(files));
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use home::home_dir;
    use walkdir::WalkDir;

    #[test]
    fn single() {
        let mut t = Tool::new("cplib");
        let tmp = "dsu";
        let out = t.run_from_module(&[tmp]);
        dbg!(out);
    }
    #[test]
    fn two() {
        let mut t = Tool::new("cplib");
        let tmp = "dsu";
        let tmp2 = "algebra";
        let out = t.run_from_module(&[tmp, tmp2]);
        dbg!(out);
    }
    #[test]
    fn file_not_exist() {
        let mut t = Tool::new("cplib");
        let tmp = "nothing";
        if let Err(e) = t.into_path(tmp) {
            assert_eq!(e.to_string(), format!("{}.rs not exist!", tmp));
        }
        let tmp = "ds";
        if let Err(e) = t.into_path(tmp) {
            assert_eq!(e.to_string(), format!("{}.rs not exist!", tmp));
        }
    }
    #[test]
    fn test_ignore() {
        let mut t = Tool::new("cplib");
        let tmp = "cmp";
        let out = t.run_from_module(&[tmp]);
        dbg!(out);
    }
    //#[test]
    //fn write() {
    //    let p = "write.rs";
    //    let mut t = Tool::new("cplib");
    //    let tmp = "dsu";
    //    t.write(p, &[tmp]);
    //}
}
