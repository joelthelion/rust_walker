extern crate rand;

use rand::thread_rng;
use rand::Rng;
use std::env;
use std::fs;
use std::option::Option;
use std::path::PathBuf;
use std::vec::Vec;

struct DirNode {
    name: PathBuf,
    children: Option<Vec<DirNode>>,
}

impl DirNode {
    fn get_children(&mut self) -> std::io::Result<&mut Vec<DirNode>> {
        match &mut self.children {
            None => {
                let mut children = Vec::new();
                for i in fs::read_dir(&self.name)? {
                    let path = i?.path();
                    if path.is_dir() {
                        let child = DirNode {
                            name: path,
                            children: None,
                        };
                        children.push(child);
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
                let child = &mut children[child_idx];
                if !pick_one(child) {
                    println!("{}", child.name.display());
                    children.remove(child_idx);
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
