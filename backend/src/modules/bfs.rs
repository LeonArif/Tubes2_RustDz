use std::collections::VecDeque;
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

pub fn bfs(tree: &Tree, query: &SearchQuery) -> TraversalResult {
    let started_at = Instant::now();
    let lca_index = tree.build_lca_index();
    let mut queue: VecDeque<usize> = VecDeque::new();
    let mut traversal_order = Vec::new();
    let mut log = Vec::new();
    let mut found_id = None;

    queue.push_back(tree.root);

    while let Some(current_node_id) = queue.pop_front() {
        let current_index = match tree.find_index_from_id(current_node_id) {
            Some(index) => index,
            None => continue,
        };

        let current_node = &tree.nodes[current_index];
        let matched = query.matches_at(tree, &lca_index, current_node_id, current_node);

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

        for child_id in current_node.children() {
            queue.push_back(*child_id);
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

#[derive(Debug)]
struct LevelVisit {
    position: usize,
    node_id: usize,
    tag: String,
    depth: usize,
    matched: bool,
    children: Vec<usize>,
}

pub fn bfs_concurrent(tree: &Tree, query: &SearchQuery) -> TraversalResult {
    let started_at = Instant::now();
    let lca_index = tree.build_lca_index();
    let mut current_level: Vec<usize> = vec![tree.root];
    let mut traversal_order = Vec::new();
    let mut log = Vec::new();
    let mut found_id = None;

    while !current_level.is_empty() {
        let available_workers = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        let workers = available_workers.min(current_level.len()).max(1);
        let chunk_size = current_level.len().div_ceil(workers);
        let lca_ref = &lca_index;
        let query_ref = query;

        let mut level_visits: Vec<LevelVisit> = Vec::new();

        thread::scope(|scope| {
            let mut handles = Vec::new();

            for (chunk_idx, chunk) in current_level.chunks(chunk_size).enumerate() {
                let base_position = chunk_idx * chunk_size;

                handles.push(scope.spawn(move || {
                    let mut partial = Vec::new();

                    for (offset, node_id) in chunk.iter().copied().enumerate() {
                        let position = base_position + offset;

                        let index = match tree.find_index_from_id(node_id) {
                            Some(i) => i,
                            None => continue,
                        };

                        let node = &tree.nodes[index];
                        partial.push(LevelVisit {
                            position,
                            node_id,
                            tag: node.tag().to_string(),
                            depth: node.depth(),
                            matched: query_ref.matches_at(tree, lca_ref, node_id, node),
                            children: node.children().clone(),
                        });
                    }

                    partial
                }));
            }

            for handle in handles {
                let mut partial = handle.join().expect("bfs worker thread panicked");
                level_visits.append(&mut partial);
            }
        });

        level_visits.sort_by_key(|visit| visit.position);

        let mut next_level = Vec::new();
        let mut stop = false;

        for visit in level_visits {
            traversal_order.push(visit.node_id);
            log.push(TraversalStep {
                step: traversal_order.len(),
                node_id: visit.node_id,
                tag: visit.tag,
                depth: visit.depth,
                matched: visit.matched,
            });

            if visit.matched {
                found_id = Some(visit.node_id);
                stop = true;
                break;
            }

            next_level.extend(visit.children);
        }

        if stop {
            break;
        }

        current_level = next_level;
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