extern crate rand;

use rand::thread_rng;
use rand::Rng;
use std::env;
use std::cell::RefCell;
use futures::stream::futures_unordered::FuturesUnordered;
use std::collections::HashMap;
use futures::stream::StreamExt;
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
    let mut nodes: HashMap<PathBuf, RefCell<NodeType>> = HashMap::new();
    loop {
        let mut path: PathBuf = PathBuf::from(path_);
        loop {
            let node = nodes.get(&path);
            match node {
                None => {
                    // spawn
                    let task = tokio::spawn(get_children(path.clone()));
                    task_queue.push(task);
                    nodes.insert(path.clone(), RefCell::new(NodeType::Pending));
                }
                Some(cell)  => {
                    match &mut *cell.borrow_mut() {
                        NodeType::Pending => {
                            break;
                        }
                        NodeType::Empty => {
                            panic!("Should not have descended onto empty node!");
                            // break; // FIXME shouldn't get here
                        }
                        NodeType::Full(children) => {
                            // descend
                            let child_idx;
                            {
                                let mut rng = thread_rng();
                                child_idx = rng.gen_range(0, children.len());
                                let current_path = &children[child_idx];
                                match nodes.get(current_path) {
                                    None => {}
                                    Some(cell) => {
                                        match &*cell.borrow() {
                                            NodeType::Empty => {
                                                children.swap_remove(child_idx);
                                            }
                                            NodeType::Pending => { break; }
                                            NodeType::Full(_) => {
                                                path = current_path.clone();
                                                break;
                                            }
                                        }
                                    }
                                };
                            }
                        }
                    }
                }
            }
        }
        if let Some(Ok(Ok((path, children)))) = task_queue.next().await {
            // Update entry
            if children.len() == 0 {
                nodes.insert(path, RefCell::new(NodeType::Empty));
            } else {
                nodes.insert(path, RefCell::new(NodeType::Full(children)));
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
