extern crate rand;

use rand::thread_rng;
use rand::Rng;
use std::env;
// use std::option::Option;
use std::path::PathBuf;
use std::vec::Vec;
use indexmap::IndexMap;
use tokio::stream::StreamExt;
use futures::future::{FutureExt, Future};
use futures::stream::futures_unordered::FuturesUnordered;
use tokio::task::JoinHandle;
use NodeType::*;

type ChildrenType = IndexMap<PathBuf, NodeType>;
type CrawlResultType = tokio::io::Result<ChildrenType>;

enum NodeType {
    Nothing,
    Pending,
    Full(DirNode),
}

struct DirNode {
    children: ChildrenType
}

async fn get_children(path: PathBuf) -> CrawlResultType {
    let mut children = IndexMap::new();
    let mut entries = tokio::fs::read_dir(path).await?;
    while let Some(i_) =  entries.next().await {
        let i = i_?;
        let is_symlink = i.file_type().await?.is_symlink();
        let path = i.path();
        if !is_symlink && path.is_dir() {
            children.insert(path, NodeType::Nothing);
        } else {
            println!("{}", path.display());
        }
    }
    Ok(children)
}


async fn random_walk(path_: &str) {
    let task_queue : FuturesUnordered<JoinHandle<CrawlResultType>> = FuturesUnordered::new();
    let mut current = NodeType::Nothing;
    let path: PathBuf = PathBuf::from(path_);
    loop {
        match &mut current {
            Nothing => {
                // spawn
                let task = tokio::spawn(get_children(path.clone()));
                task_queue.push(task);
            }
            Pending => { break; }
            Full(node) => {
                // descend
                let child_idx;
                {
                    let mut rng = thread_rng();
                    child_idx = rng.gen_range(0, node.children.len());
                    let (path, current) = node.children.get_index_mut(child_idx).unwrap();
                }
            }
        }
    }
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let dir = if args.len() >= 2 { &args[1] } else { "." };
    random_walk(dir).await;
    Ok(())
}
