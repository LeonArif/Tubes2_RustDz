use std::collections::VecDeque;
use crate::modules::domnode;
use crate::modules::tree;

pub fn bfs(tree: &Tree) -> Vec<DomNode>{
    let mut queue = VecDeque<usize>::new();
    let mut hasil: Vec<DomNode> = Vec::new();
    
    queue.push_back(tree.root);

    while !queue.is_empty(){
        let current_node_id = match queue.pop_front(){
            Some(id) => id,
            None => None
        };
        let mut currnet_index = find_index_from_id(current_node_id);

        let current_child = tree.nodes[current_index].children();
        for i in current_child{
            queue.push_back(*i);
        }
    }

    
}