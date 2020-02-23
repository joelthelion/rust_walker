extern crate rand;

use rand::thread_rng;
use rand::Rng;
use std::env;
use std::fs;
use std::option::Option;
use std::path::PathBuf;
use std::vec::Vec;
// use std::collections::HashMap;
use indexmap::IndexMap;

struct DirNode {
    children: Option<IndexMap<PathBuf,DirNode>>,
}

impl DirNode {
    fn get_children(&mut self, path: &PathBuf) -> std::io::Result<&mut IndexMap<PathBuf, DirNode>> {
        match &mut self.children {
            None => {
                let mut children = IndexMap::new();
                for i_ in fs::read_dir(path)? {
                    let i = i_?;
                    let is_symlink = i.file_type()?.is_symlink();
                    let path = i.path();
                    if !is_symlink && path.is_dir() {
                        let child = DirNode {
                            children: None,
                        };
                        children.insert(path, child);
                    } else {
                        println!("{}", path.display());
                    }
                }
                self.children = Some(children);
                Ok(self.children.as_mut().unwrap())
            }
            Some(_) => Ok(self.children.as_mut().unwrap()),
        }
    }
}

enum PickOneResult {
    OK,
    Empty,
    Error(std::io::Error)
}

fn pick_one(path: &PathBuf, node: &mut DirNode) -> PickOneResult {
    let children_ = node.get_children(path);
    match children_ {
        Ok(children) => {
            if children.is_empty() {
                PickOneResult::Empty
            } else {
                let mut rng = thread_rng();
                let child_idx = rng.gen_range(0, children.len());
                let (child_name, child) = children.get_index_mut(child_idx).unwrap();
                match pick_one(child_name, child) {
                    PickOneResult::OK => {}
                    PickOneResult::Empty => {
                        println!("{}", child_name.display());
                        children.swap_remove_index(child_idx);
                    }
                    PickOneResult::Error(err) => {
                        eprintln!("{} : {}", child_name.display(), err);
                        println!("{}", child_name.display());
                        children.swap_remove_index(child_idx);
                    }
                }
                PickOneResult::OK
            }
        }
        Err(error) => PickOneResult::Error(error)
    }
}

fn random_walk(path_: &str) {
    let mut node = DirNode {
        children: None,
    };
    let path : PathBuf = PathBuf::from(path_);
    while let PickOneResult::OK =  pick_one(&path, &mut node) {
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let dir = if args.len() >= 2 { &args[1] } else { "." };
    random_walk(dir);
}
