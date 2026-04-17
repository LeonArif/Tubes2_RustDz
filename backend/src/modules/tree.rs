use crate::modules::domnode;
use std::collections::HashMap;
#[derive(Debug, Clone)]

pub struct Tree{
    pub nodes: Vec<DomNode>,
    pub root: usize,
    pub id_to_index: HashMap<usize, usize>,

}

impl Tree{

    fn new(){
        Tree{
            nodes: Vec::new(),
            root: 0,
            id_to_index: HashMap::new(),
        }
    }

    fn find_index_from_id(&self, id:usize) -> Option(usize) {
        return self.id_to_index.get(&id).copied();
    }

    fn insert_node(&mut self, node: DomNode){
        let id = node.id();
        if self.id_to_index.contains_key(&id){
            return;
        }
        let index = self.nodes.len();
        self.nodes.push(node);
        self.id_to_index.insert(id, index);
    }

    fn add_root(&mut self, tag: String, attrs: Vec<(String, String)>){
        if self.id_to_index.contains_key(&0){
            return;
        }
        self.insert_node(DomNode::new(0,tag, None, attrs, 0));
        self.root = 0;
    }

    fn add_child(&mut self, id: usize, tag: String, parent_id:usize, attrs: Vec<(String, String)>){
        if self.id_to_index.contains_key(&id){
            return;
        }

        let parent_idx = match self.find_index_from_id(parent_id){
            Some(idx) => idx,
            None => return,
        };

        let parent_depth = self.nodes[parent_idx].depth();

        self.insert_node(DomNode::new(id,tag,Some(parent_id),attrs,parent_depth+1));

    }
}

