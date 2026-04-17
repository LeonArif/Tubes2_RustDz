use std::thread;
use std::time::Instant;
use crate::modules::domnode::DomNode;
use crate::modules::selector::{query_matches_node, SelectorQuery};
use crate::modules::tree::{LcaBinaryLifting, Tree};

#[derive(Debug, Clone)]
pub enum SearchQuery {
    Tag(String),
    Attr(String, String),
    TagAndAttr { tag: String, key: String, value: String },
    Selector(SelectorQuery),
}

impl SearchQuery {
    pub fn matches_at(
        &self,
        tree: &Tree,
        lca: &LcaBinaryLifting,
        node_id: usize,
        node: &DomNode,
    ) -> bool {
        match self {
            SearchQuery::Tag(tag) => node.tag() == tag,
            SearchQuery::Attr(key, value) => {
                node.attrs().iter().any(|(k, v)| k == key && v == value)
            }
            SearchQuery::TagAndAttr { tag, key, value } => {
                node.tag() == tag
                    && node.attrs().iter().any(|(k, v)| k == key && v == value)
            }
            SearchQuery::Selector(selector_query) => {
                query_matches_node(tree, lca, node_id, selector_query)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TraversalStep {
    pub step: usize,
    pub node_id: usize,
    pub tag: String,
    pub depth: usize,
    pub matched: bool,
}

#[derive(Debug, Clone)]
pub struct TraversalMetrics {
    pub visited_nodes: usize,
    pub elapsed_ms: u128,
    pub max_depth: usize,
}

#[derive(Debug, Clone)]
pub struct TraversalResult {
    pub found: Option<DomNode>,
    pub found_id: Option<usize>,
    pub traversal_order: Vec<usize>,
    pub path_to_found: Vec<usize>,
    pub log: Vec<TraversalStep>,
    pub metrics: TraversalMetrics,
}

pub fn path_to_root(tree: &Tree, target_id: usize) -> Vec<usize> {
    let mut path = Vec::new();
    let mut current = Some(target_id);

    while let Some(node_id) = current {
        path.push(node_id);
        let index = match tree.find_index_from_id(node_id) {
            Some(i) => i,
            None => break,
        };
        current = tree.nodes[index].parent();
    }

    path.reverse();
    path
}

fn run_dfs_from_start(
    tree: &Tree,
    lca: &LcaBinaryLifting,
    query: &SearchQuery,
    start_node_id: usize,
) -> (Vec<usize>, Vec<TraversalStep>, Option<usize>) {
    let mut stack: Vec<usize> = vec![start_node_id];
    let mut traversal_order = Vec::new();
    let mut log = Vec::new();
    let mut found_id = None;

    while let Some(current_node_id) = stack.pop() {
        let current_index = match tree.find_index_from_id(current_node_id) {
            Some(index) => index,
            None => continue,
        };

        let current_node = &tree.nodes[current_index];
        let matched = query.matches_at(tree, lca, current_node_id, current_node);

        traversal_order.push(current_node_id);
        log.push(TraversalStep {
            step: traversal_order.len(),
            node_id: current_node_id,
            tag: current_node.tag().to_string(),
            depth: current_node.depth(),
            matched,
        });

        if matched {
            found_id = Some(current_node_id);
            break;
        }

        for child_id in current_node.children().iter().rev() {
            stack.push(*child_id);
        }
    }

    (traversal_order, log, found_id)
}

pub fn dfs(tree: &Tree, query: &SearchQuery) -> TraversalResult {
    let started_at = Instant::now();
    let lca_index = tree.build_lca_index();

    let (traversal_order, mut log, found_id) =
        run_dfs_from_start(tree, &lca_index, query, tree.root);

    for (idx, step) in log.iter_mut().enumerate() {
        step.step = idx + 1;
    }

    let found = found_id.and_then(|id| {
        tree.find_index_from_id(id)
            .and_then(|index| tree.nodes.get(index).cloned())
    });

    let path_to_found = found_id
        .map(|id| path_to_root(tree, id))
        .unwrap_or_default();

    TraversalResult {
        found,
        found_id,
        traversal_order: traversal_order.clone(),
        path_to_found,
        log,
        metrics: TraversalMetrics {
            visited_nodes: traversal_order.len(),
            elapsed_ms: started_at.elapsed().as_millis(),
            max_depth: tree.max_depth(),
        },
    }
}

#[derive(Debug)]
struct SubtreeVisit {
    position: usize,
    traversal_order: Vec<usize>,
    log: Vec<TraversalStep>,
    found_id: Option<usize>,
}

pub fn dfs_concurrent(tree: &Tree, query: &SearchQuery) -> TraversalResult {
    let started_at = Instant::now();
    let lca_index = tree.build_lca_index();

    let root_index = match tree.find_index_from_id(tree.root) {
        Some(index) => index,
        None => {
            return TraversalResult {
                found: None,
                found_id: None,
                traversal_order: Vec::new(),
                path_to_found: Vec::new(),
                log: Vec::new(),
                metrics: TraversalMetrics {
                    visited_nodes: 0,
                    elapsed_ms: started_at.elapsed().as_millis(),
                    max_depth: 0,
                },
            };
        }
    };

    let root_node = &tree.nodes[root_index];
    let root_matched = query.matches_at(tree, &lca_index, tree.root, root_node);

    let mut traversal_order = vec![tree.root];
    let mut log = vec![TraversalStep {
        step: 1,
        node_id: tree.root,
        tag: root_node.tag().to_string(),
        depth: root_node.depth(),
        matched: root_matched,
    }];

    let mut found_id = if root_matched { Some(tree.root) } else { None };

    if found_id.is_none() {
        let root_children = root_node.children().clone();
        let available_workers = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        let workers = available_workers.min(root_children.len()).max(1);
        let chunk_size = root_children.len().div_ceil(workers);

        let mut subtree_visits: Vec<SubtreeVisit> = Vec::new();

        thread::scope(|scope| {
            let mut handles = Vec::new();

            for (chunk_idx, chunk) in root_children.chunks(chunk_size).enumerate() {
                let base_position = chunk_idx * chunk_size;
                let lca_ref = &lca_index;
                let query_ref = query;

                handles.push(scope.spawn(move || {
                    let mut partial = Vec::new();

                    for (offset, node_id) in chunk.iter().copied().enumerate() {
                        let position = base_position + offset;
                        let (sub_order, sub_log, sub_found_id) =
                            run_dfs_from_start(tree, lca_ref, query_ref, node_id);

                        partial.push(SubtreeVisit {
                            position,
                            traversal_order: sub_order,
                            log: sub_log,
                            found_id: sub_found_id,
                        });
                    }

                    partial
                }));
            }

            for handle in handles {
                let mut partial = handle.join().expect("dfs worker thread panicked");
                subtree_visits.append(&mut partial);
            }
        });

        subtree_visits.sort_by_key(|visit| visit.position);

        for visit in subtree_visits {
            let start_step = traversal_order.len();

            traversal_order.extend(visit.traversal_order);
            for (idx, mut step) in visit.log.into_iter().enumerate() {
                step.step = start_step + idx + 1;
                log.push(step);
            }

            if found_id.is_none() {
                found_id = visit.found_id;
                if found_id.is_some() {
                    break;
                }
            }
        }
    }

    let found = found_id.and_then(|id| {
        tree.find_index_from_id(id)
            .and_then(|index| tree.nodes.get(index).cloned())
    });

    let path_to_found = found_id
        .map(|id| path_to_root(tree, id))
        .unwrap_or_default();

    TraversalResult {
        found,
        found_id,
        traversal_order: traversal_order.clone(),
        path_to_found,
        log,
        metrics: TraversalMetrics {
            visited_nodes: traversal_order.len(),
            elapsed_ms: started_at.elapsed().as_millis(),
            max_depth: tree.max_depth(),
        },
    }
}

