#[derive(Debug, Clone)]
pub struct DomNode {
    id: usize,
    tag: String,
    parent: Option<usize>,
    children: Vec<usize>,
    attrs: Vec<(String, String)>,
    depth: usize
}

impl DomNode {
    pub fn new(id: usize,tag: String,parent: Option<usize>,attrs: Vec<(String, String)>,depth: usize) -> Self {
        DomNode {
            id,
            tag,
            parent,
            children: Vec::new(),
            attrs,
            depth
        }
    }
    
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn parent(&self) -> Option<usize> {
        self.parent
    }

    pub fn children(&self) -> &Vec<usize> {
        &self.children
    }

    pub fn attrs(&self) -> &Vec<(String, String)> {
        &self.attrs
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn add_child(&mut self, child_id: usize) {
        self.children.push(child_id);
    }

}