use crate::modules::domnode::DomNode;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Tree {
    nodes: Vec<DomNode>,
    root: usize,
    id_to_index: HashMap<usize, usize>,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            root: 0,
            id_to_index: HashMap::new(),
        }
    }

    pub fn from_nodes(nodes: Vec<DomNode>) -> Result<Self, String> {
        let mut tree = Self::new();

        for node in nodes {
            let id = node.id();
            if tree.id_to_index.contains_key(&id) {
                return Err(format!("Duplicate node id: {}", id));
            }

            tree.id_to_index.insert(id, tree.nodes.len());
            tree.nodes.push(node);
        }

        if tree.id_to_index.contains_key(&0) {
            tree.root = 0;
        }

        Ok(tree)
    }

    pub fn add_root(
        &mut self,
        tag: String,
        attrs: Vec<(String, String)>,
    ) -> Result<(), String> {
        if self.id_to_index.contains_key(&0) {
            return Err("Root already exists".to_string());
        }

        self.insert_node(DomNode::new(0, tag, None, attrs, 0));
        self.root = 0;
        Ok(())
    }

    pub fn add_child(
        &mut self,
        id: usize,
        tag: String,
        parent_id: usize,
        attrs: Vec<(String, String)>,
    ) -> Result<(), String> {
        if self.id_to_index.contains_key(&id) {
            return Err(format!("Node id {} already exists", id));
        }

        let parent_index = self
            .find_index_from_id(parent_id)
            .ok_or_else(|| format!("Parent id {} not found", parent_id))?;

        let parent_depth = self.nodes[parent_index].depth();
        let node = DomNode::new(id, tag, Some(parent_id), attrs, parent_depth + 1);

        self.insert_node(node)?;
        self.nodes[parent_index].add_child(id);
        Ok(())
    }

    pub fn find_index_from_id(&self, id: usize) -> Option<usize> {
        self.id_to_index.get(&id).copied()
    }

    pub fn root(&self) -> usize {
        self.root
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn max_depth(&self) -> usize {
        self.nodes.iter().map(DomNode::depth).max().unwrap_or(0)
    }

    pub fn get_node(&self, id: usize) -> Option<&DomNode> {
        self.find_index_from_id(id)
            .and_then(|index| self.nodes.get(index))
    }

    pub fn get_children(&self, id: usize) -> Option<&Vec<usize>> {
        self.get_node(id).map(DomNode::children)
    }

    pub fn get_parent(&self, id: usize) -> Option<usize> {
        self.get_node(id).and_then(DomNode::parent)
    }

    pub fn nodes(&self) -> &Vec<DomNode> {
        &self.nodes
    }

    fn insert_node(&mut self, node: DomNode) -> Result<(), String> {
        let id = node.id();
        if self.id_to_index.contains_key(&id) {
            return Err(format!("Duplicate node id: {}", id));
        }

        let index = self.nodes.len();
        self.nodes.push(node);
        self.id_to_index.insert(id, index);
        Ok(())
    }
}

pub fn bfs(tree: &Tree) -> Vec<DomNode>{
    let mut queue: VecDeque<usize> = VecDeque::new();
    let mut hasil: Vec<DomNode> = Vec::new();

    queue.push_back(tree.root);

    while let Some(current_id) = queue.pop_front() {
        let current_index = match tree.id_to_index.get(&current_id) {
            Some(index) => *index,
            None => continue,
        };

        let current_node = &tree.nodes[current_index];
        hasil.push(current_node.clone());

        for child_id in current_node.children() {
            queue.push_back(*child_id);
        }
    }

    hasil
}