extern crate rand;

use rand::thread_rng;
use rand::Rng;
use std::env;
// use std::option::Option;
use futures::future::{Future, FutureExt};
use futures::stream::futures_unordered::FuturesUnordered;
use futures::stream::{Stream, StreamExt};
use std::collections::HashMap;
use std::path::PathBuf;
use std::vec::Vec;
use tokio::task::JoinHandle;

type ChildrenType = Vec<PathBuf>;
type CrawlResultType = tokio::io::Result<(PathBuf, ChildrenType)>;

#[derive(Debug)]
enum NodeType {
    Pending,
    Empty,
    Full(ChildrenType),
}

async fn get_children(path: PathBuf) -> CrawlResultType {
    let mut children = Vec::new();
    let mut entries = tokio::fs::read_dir(&path).await?;
    while let Some(i_) = entries.next().await {
        let i = i_?;
        let is_symlink = i.file_type().await?.is_symlink();
        let path = i.path();
        if !is_symlink && path.is_dir() {
            children.push(path);
        } else {
            println!("{}", path.display());
        }
    }
    Ok((path, children))
}

async fn random_walk(path_: &str) {
    let mut task_queue: FuturesUnordered<JoinHandle<CrawlResultType>> = FuturesUnordered::new();
    let mut nodes: HashMap<PathBuf, NodeType> = HashMap::new();
    loop {
        let mut path: PathBuf = PathBuf::from(path_);
        loop {
            match nodes.get(&path) {
                None => {
                    // spawn
                    let task = tokio::spawn(get_children(path.clone()));
                    task_queue.push(task);
                    nodes.insert(path.clone(), NodeType::Pending);
                }
                Some(NodeType::Pending) => {
                    break;
                }
                Some(NodeType::Empty) => {
                    break;
                } // FIXME shouldn't get here
                Some(NodeType::Full(children)) => {
                    // descend
                    let child_idx;
                    {
                        let mut rng = thread_rng();
                        child_idx = rng.gen_range(0, children.len());
                        path = children[child_idx].clone();
                    }
                }
            }
        }
        if let Some(Ok(Ok((path, children)))) = task_queue.next().await {
            // Update entry
            if children.len() == 0 {
                nodes.insert(path, NodeType::Empty);
            } else {
                nodes.insert(path, NodeType::Full(children));
            }
        } else {
            // If nothing in queue. FIXME handle crawling errors
            break;
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
