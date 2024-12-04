#[derive(Default)]
pub struct OctreeNode {
    pub block_type: Option<u8>,
    pub children: Option<[Box<OctreeNode>; 8]>,
    pub needs_generation: bool, // New field to track lazy generation
}


impl OctreeNode {
    pub fn block_type(&self) -> Option<u8> {
        self.block_type
    }

    pub fn children(&self) -> Option<&[Box<OctreeNode>; 8]> {
        self.children.as_ref()
    }
    
}
