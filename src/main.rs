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
    name: PathBuf,
    children: Option<IndexMap<PathBuf,DirNode>>,
}

impl DirNode {
    fn get_children(&mut self) -> std::io::Result<&mut IndexMap<PathBuf, DirNode>> {
        match &mut self.children {
            None => {
                let mut children = IndexMap::new();
                for i_ in fs::read_dir(&self.name)? {
                    let i = i_?;
                    let is_symlink = i.file_type()?.is_symlink();
                    let path = i.path();
                    if !is_symlink && path.is_dir() {
                        let child = DirNode {
                            name: path,
                            children: None,
                        };
                        children.insert(child.name.clone(), child);
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

fn pick_one(node: &mut DirNode) -> bool {
    let children_ = node.get_children();
    match children_ {
        Ok(children) => {
            if children.is_empty() {
                println!("{}", node.name.display());
                false
            } else {
                let mut rng = thread_rng();
                let child_idx = rng.gen_range(0, children.len());
                let (_, child) = children.get_index_mut(child_idx).unwrap();
                if !pick_one(child) {
                    println!("{}", child.name.display());
                    children.swap_remove_index(child_idx);
                }
                true
            }
        }
        Err(error) => {
            eprintln!("{} : {}", node.name.display(), error);
            println!("{}", node.name.display());
            false
        }
    }
}

fn random_walk(path: &str) {
    let mut node = DirNode {
        name: PathBuf::from(path),
        children: None,
    };
    loop {
        if !pick_one(&mut node) {
            break;
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let dir = if args.len() >= 2 { &args[1] } else { "." };
    random_walk(dir);
}
