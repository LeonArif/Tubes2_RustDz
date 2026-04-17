use crate::modules::domnode::DomNode;
use std::collections::HashMap;
#[derive(Debug, Clone)]

pub struct Tree{
    pub nodes: Vec<DomNode>,
    pub root: usize,
    pub id_to_index: HashMap<usize, usize>,

}

#[derive(Debug, Clone)]
pub struct LcaBinaryLifting {
    up: Vec<Vec<Option<usize>>>,
    depth: Vec<usize>,
}

impl Tree{

    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            root: 0,
            id_to_index: HashMap::new(),
        }
    }

    pub fn find_index_from_id(&self, id:usize) -> Option<usize> {
        return self.id_to_index.get(&id).copied();
    }

    pub fn insert_node(&mut self, node: DomNode){
        let id = node.id();
        if self.id_to_index.contains_key(&id){
            return;
        }
        let index = self.nodes.len();
        self.nodes.push(node);
        self.id_to_index.insert(id, index);
    }

    pub fn add_root(&mut self, tag: String, attrs: Vec<(String, String)>){
        if self.id_to_index.contains_key(&0){
            return;
        }
        self.insert_node(DomNode::new(0,tag, None, attrs, 0));
        self.root = 0;
    }

    pub fn add_child(&mut self, id: usize, tag: String, parent_id:usize, attrs: Vec<(String, String)>){
        if self.id_to_index.contains_key(&id){
            return;
        }

        let parent_idx = match self.find_index_from_id(parent_id){
            Some(idx) => idx,
            None => return,
        };

        let parent_depth = self.nodes[parent_idx].depth();

        self.insert_node(DomNode::new(id,tag,Some(parent_id),attrs,parent_depth+1));
        self.nodes[parent_idx].add_child(id);

    }

    pub fn max_depth(&self) -> usize {
        self.nodes.iter().map(|node| node.depth()).max().unwrap_or(0)
    }

    pub fn build_lca_index(&self) -> LcaBinaryLifting {
        LcaBinaryLifting::build(self)
    }
}

impl LcaBinaryLifting {
    pub fn build(tree: &Tree) -> Self {
        let n = tree.nodes.len();
        if n == 0 {
            return Self {
                up: Vec::new(),
                depth: Vec::new(),
            };
        }

        let mut max_log = 1usize;
        while (1usize << max_log) <= n {
            max_log += 1;
        }

        let mut up = vec![vec![None; max_log]; n];
        let mut depth = vec![0usize; n];
        let mut visited = vec![false; n];

        if let Some(root_idx) = tree.find_index_from_id(tree.root) {
            let mut stack = vec![root_idx];
            visited[root_idx] = true;

            while let Some(parent_idx) = stack.pop() {
                let parent_depth = depth[parent_idx];

                for child_id in tree.nodes[parent_idx].children() {
                    let child_idx = match tree.find_index_from_id(*child_id) {
                        Some(index) => index,
                        None => continue,
                    };

                    if visited[child_idx] {
                        continue;
                    }

                    visited[child_idx] = true;
                    up[child_idx][0] = Some(parent_idx);
                    depth[child_idx] = parent_depth + 1;
                    stack.push(child_idx);
                }
            }
        }

        for node_idx in 0..n {
            if visited[node_idx] {
                continue;
            }

            up[node_idx][0] = tree.nodes[node_idx]
                .parent()
                .and_then(|parent_id| tree.find_index_from_id(parent_id));
        }

        for jump in 1..max_log {
            for node_idx in 0..n {
                up[node_idx][jump] = up[node_idx][jump - 1]
                    .and_then(|mid_idx| up[mid_idx][jump - 1]);
            }
        }

        Self { up, depth }
    }

    pub fn lca_index(&self, a_idx: usize, b_idx: usize) -> Option<usize> {
        if self.up.is_empty() {
            return None;
        }

        if a_idx >= self.up.len() || b_idx >= self.up.len() {
            return None;
        }

        let mut a = a_idx;
        let mut b = b_idx;

        if self.depth[a] < self.depth[b] {
            std::mem::swap(&mut a, &mut b);
        }

        let diff = self.depth[a] - self.depth[b];
        for jump in (0..self.up[0].len()).rev() {
            if ((diff >> jump) & 1) == 1 {
                a = self.up[a][jump]?;
            }
        }

        if a == b {
            return Some(a);
        }

        for jump in (0..self.up[0].len()).rev() {
            let next_a = self.up[a][jump];
            let next_b = self.up[b][jump];

            if next_a.is_some() && next_a != next_b {
                a = next_a?;
                b = next_b?;
            }
        }

        self.up[a][0]
    }

    pub fn lca_id(&self, tree: &Tree, a_id: usize, b_id: usize) -> Option<usize> {
        let a_idx = tree.find_index_from_id(a_id)?;
        let b_idx = tree.find_index_from_id(b_id)?;
        let lca_idx = self.lca_index(a_idx, b_idx)?;
        Some(tree.nodes[lca_idx].id())
    }

    pub fn is_ancestor_id(&self, tree: &Tree, ancestor_id: usize, node_id: usize) -> bool {
        matches!(self.lca_id(tree, ancestor_id, node_id), Some(id) if id == ancestor_id)
    }
}

