extern crate rand;

use futures::stream::futures_unordered::FuturesUnordered;
use futures::stream::StreamExt;
use rand::thread_rng;
use rand::Rng;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
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
    let orig_path: PathBuf = PathBuf::from(path_);
    'outer: loop {
        let mut path: PathBuf = orig_path.clone();
        'descend: loop {
            let maybe_node = nodes.get(&path);
            match maybe_node {
                None => {
                    // spawn
                    let task = tokio::spawn(get_children(path.clone()));
                    task_queue.push(task);
                    nodes.insert(path.clone(), RefCell::new(NodeType::Pending));
                    continue 'outer;
                }
                Some(cell) => {
                    let node = &mut *cell.borrow_mut();
                    match node {
                        NodeType::Pending => {
                            break;
                        }
                        NodeType::Empty => {
                            println!("panic: {:?}", path);
                            panic!("Should not have descended onto empty node!");
                        }
                        NodeType::Full(children) => {
                            let mut rng = thread_rng();
                            loop {
                                if children.is_empty() {
                                    println!("{}", path.display());
                                    if path == orig_path {
                                        *node = NodeType::Empty;
                                        break 'descend;
                                    } else {
                                        *node = NodeType::Empty;
                                        continue 'outer;
                                    }
                                }
                                let child_idx;
                                child_idx = rng.gen_range(0, children.len());
                                let current_path = &children[child_idx];
                                match nodes.get(current_path) {
                                    None => {
                                        path = current_path.clone();
                                        continue 'descend;
                                    }
                                    Some(cell) => match &*cell.borrow() {
                                        NodeType::Empty => {
                                            children.swap_remove(child_idx);
                                        }
                                        NodeType::Pending => {
                                            break 'descend;
                                        }
                                        NodeType::Full(_) => {
                                            path = current_path.clone();
                                            continue 'descend;
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Some(join_result) = task_queue.next().await {
            let crawl_result = join_result.unwrap();
            match crawl_result {
                Err(err) => {
                    eprintln!("Crawling error: {}", err);
                }
                Ok((path, children)) => {
                    // Update entry
                    if children.len() == 0 {
                        println!("{}", path.display());
                        nodes.insert(path, RefCell::new(NodeType::Empty));
                    } else {
                        nodes.insert(path, RefCell::new(NodeType::Full(children)));
                    }
                }
            };
        } else {
            // If nothing in queue, we should be done
            break;
        }
    }
}

#[tokio::main(max_threads=12)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let dir = if args.len() >= 2 { &args[1] } else { "./" };
    random_walk(dir).await;
    Ok(())
}
