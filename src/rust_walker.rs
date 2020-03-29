//! Asynchronous randomized large filesystem explorer
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

/// Node status for the main tree.
/// * Pending: waiting for asynchronous directory listing to complete
/// * Empty: either originally empty or entirely processed
/// * Full: ready for exploration
#[derive(Debug)]
enum NodeType {
    Pending,
    Empty,
    Full(ChildrenType),
}

enum TaskOutput {
    DirEntry((PathBuf, tokio::fs::ReadDir)),
    Child((PathBuf, tokio::fs::ReadDir, PathBuf)),
    Nothing,
}

struct Walker {
    /// Unordered task queue for directory listing jobs
    task_queue: FuturesUnordered<JoinHandle<tokio::io::Result<TaskOutput>>>,
    /// Visited paths and their status
    nodes: HashMap<PathBuf, RefCell<NodeType>>,
}

impl Walker {
    /// Create a new random walker
    fn new() -> Walker {
        Walker {
            task_queue: FuturesUnordered::new(),
            nodes: HashMap::new(),
        }
    }
    /// Randomly walk through a directory until all paths are traversed
    async fn walk(&mut self, path: &PathBuf) {
        loop {
            // Sample directories while we can
            self.walk_until_pending(path);
            // Retrieve the output of one task and update the tree
            if !self.process_one_task().await {
                break;
            }
        }
    }
    /// Repeatedly descend through the tree until we reach a pending node
    fn walk_until_pending(&mut self, orig_path: &PathBuf) {
        'outer: loop {
            let mut path = orig_path.clone();
            'descend: loop {
                let maybe_node = self.nodes.get(&path);
                match maybe_node {
                    None => {
                        // spawn
                        let task = tokio::spawn(read_dir(path.clone()));
                        self.task_queue.push(task);
                        self.nodes
                            .insert(path.clone(), RefCell::new(NodeType::Pending));
                        continue 'outer;
                    }
                    Some(cell) => {
                        let node = &mut *cell.borrow_mut();
                        match node {
                            NodeType::Pending => {
                                return;
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
                                        if path == *orig_path {
                                            *node = NodeType::Empty;
                                            return;
                                        } else {
                                            *node = NodeType::Empty;
                                            continue 'outer;
                                        }
                                    }
                                    let child_idx;
                                    child_idx = rng.gen_range(0, children.len());
                                    let current_path = &children[child_idx];
                                    match self.nodes.get(current_path) {
                                        None => {
                                            path = current_path.clone();
                                            continue 'descend;
                                        }
                                        Some(cell) => match &*cell.borrow() {
                                            NodeType::Empty => {
                                                children.swap_remove(child_idx);
                                            }
                                            NodeType::Pending => {
                                                return;
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
        }
    }
    /// Retrieve the output of one task and update the tree
    async fn process_one_task(&mut self) -> bool {
        if let Some(join_result) = self.task_queue.next().await {
            let crawl_result = join_result.unwrap();
            match crawl_result {
                Err(err) => {
                    eprintln!("Crawling error: {}", err);
                }
                Ok(task_output) => {
                    match task_output {
                        TaskOutput::DirEntry((path, entries)) => {
                            let task = tokio::spawn(get_next_child(path, entries));
                            self.task_queue.push(task);
                        }
                        TaskOutput::Child((node_path, entries, child_path)) => {
                            match self.nodes.get(&node_path) {
                                Some(cell) => {
                                    let mut node = cell.borrow_mut();
                                    match &mut *node {
                                        NodeType::Pending => {
                                            let mut children = Vec::new();
                                            children.push(child_path);
                                            *node = NodeType::Full(children);
                                        }
                                        NodeType::Full(children) => {
                                            children.push(child_path);
                                        }
                                        NodeType::Empty => {
                                            panic!("Nodes should not be marked as empty before they are fully processed")
                                        }
                                    }
                                }
                                None => {
                                    panic!("Should be pending or full")
                                }
                            }
                            let next_task = tokio::spawn(get_next_child(node_path, entries));
                            self.task_queue.push(next_task);
                        }
                        TaskOutput::Nothing => {
                            println!("Nothing left to do for this job!!");
                        }
                    }
                }
            };
            true
        } else {
            // If nothing in queue, we should be done
            false
        }
    }
}

#[tokio::main(max_threads = 12)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let dir = PathBuf::from(if args.len() >= 2 { &args[1] } else { "./" });
    let mut walker = Walker::new();
    walker.walk(&dir).await;
    Ok(())
}

type ChildrenType = Vec<PathBuf>;


async fn read_dir(path: PathBuf) -> tokio::io::Result<TaskOutput> {
    let entries = tokio::fs::read_dir(&path).await?;
    Ok(TaskOutput::DirEntry((path, entries)))
}

async fn get_next_child(path: PathBuf, mut entries: tokio::fs::ReadDir) -> tokio::io::Result<TaskOutput>
{
    loop {
        return match entries.next_entry().await? {
            Some(entry) => {
                let is_symlink = entry.file_type().await?.is_symlink();
                let child = entry.path();
                if !is_symlink && child.is_dir() {
                    Ok(TaskOutput::Child((path, entries, child)))
                } else {
                    println!("{}", child.display());
                    continue
                }
            }
            None => Ok(TaskOutput::Nothing)
        }
    }
}
