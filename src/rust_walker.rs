extern crate rand;

use rand::thread_rng;
use rand::Rng;
use std::env;
// use std::option::Option;
use std::path::PathBuf;
use std::vec::Vec;
use indexmap::IndexMap;
use tokio::stream::StreamExt;
use futures::future::{BoxFuture, FutureExt, Future, Shared};

type ChildrenType = IndexMap<PathBuf, DirNode>;
// type ChildrenFuture = u64;

enum NodeType {
        Nothing,
        Pending(Box<dyn Future<Output = tokio::io::Result<ChildrenType>>>), // FIXME
        Children(ChildrenType),
    }


struct DirNode {
    children: NodeType
}

impl DirNode {
    async fn _get_children(&mut self, path: &PathBuf) -> tokio::io::Result<ChildrenType> {
        let mut children = IndexMap::new();
        let mut entries = tokio::fs::read_dir(path).await?;
        while let Some(i_) =  entries.next().await {
            let i = i_?;
            let is_symlink = i.file_type().await?.is_symlink();
            let path = i.path();
            if !is_symlink && path.is_dir() {
                let child = DirNode { children: NodeType::Nothing };
                children.insert(path, child);
            } else {
                println!("{}", path.display());
            }
        }
        Ok(children)
    }
    async fn get_children(&mut self, path: &PathBuf) -> tokio::io::Result<NodeType> {
        match &mut self.children {
            NodeType::Nothing => {
                let children = self._get_children(path);
                self.children = NodeType::Pending(Box::new(children.shared())); //FIXME race condition
                Ok(self.children)
            }
            NodeType::Pending(f) => f.await,
            NodeType::Children(_) => Ok(self.children.as_mut().unwrap()),
        }
    }
}

enum PickOneResult {
    OK,
    Empty,
    Error(std::io::Error),
}

fn pick_one<'a>(path: &'a PathBuf, node: &'a mut DirNode) -> BoxFuture<'a, PickOneResult> {
    async move {
        let children_ = node.get_children(path).await;
        match children_ {
            Ok(children) => {
                if children.is_empty() {
                    PickOneResult::Empty
                } else {
                    let child_idx;
                    {
                        let mut rng = thread_rng();
                        child_idx = rng.gen_range(0, children.len());
                    }
                    let (child_name, child) = children.get_index_mut(child_idx).unwrap();
                    match pick_one(child_name, child).await {
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
            Err(error) => PickOneResult::Error(error),
        }
    }.boxed()
}

async fn random_walk(path_: &str) {
    let mut node = DirNode { children: None };
    let path: PathBuf = PathBuf::from(path_);
    // while let PickOneResult::OK = pick_one(&path, &mut node).await {}
    let nodeb = node.clone();
    while let PickOneResult::OK = pick_one(&path, &mut node).await {}
}

// fn main() {
//     let args: Vec<String> = env::args().collect();
//     let dir = if args.len() >= 2 { &args[1] } else { "." };
//     random_walk(dir);
// }


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let dir = if args.len() >= 2 { &args[1] } else { "." };
    random_walk(dir).await;
    Ok(())
}
