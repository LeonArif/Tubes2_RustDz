use std::time::Instant;

use crate::modules::bfs::{path_to_root, SearchQuery, TraversalMetrics, TraversalResult, TraversalStep};
use crate::modules::tree::Tree;

pub fn dfs(tree: &Tree, query: &SearchQuery) -> TraversalResult {
	let started_at = Instant::now();
	let lca_index = tree.build_lca_index();
	let mut stack: Vec<usize> = vec![tree.root];
	let mut traversal_order = Vec::new();
	let mut log = Vec::new();
	let mut found_id = None;

	while let Some(current_node_id) = stack.pop() {
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

		for child_id in current_node.children().iter().rev() {
			stack.push(*child_id);
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
