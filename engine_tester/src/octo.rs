use rand::prelude::*;
use std::collections::{HashMap, HashSet};

pub struct OctreeNode {
    block_types: Vec<u8>,            // Possible block types for this node
    children: Option<[Box<OctreeNode>; 8]>, // Subdivided children (None if collapsed or homogeneous)
}

impl OctreeNode {
    pub fn block_type(&self) -> Vec<u8> {
        self.block_types.clone()
    }

    pub fn children(&self) -> Option<&[Box<OctreeNode>; 8]> {
        self.children.as_ref()
    }

    pub fn is_collapsed(&self) -> bool {
        !self.block_types.is_empty() && self.block_types.len() == 1
    }

    pub fn collapse(&mut self, rng: &mut StdRng) {
        if !self.is_collapsed() {
            let chosen = rng.gen_range(0..self.block_types.len());
            self.block_types = vec![self.block_types[chosen]];
        }
    }
}

pub struct TileRules {
    // Rules for block adjacency in 3D (e.g., north, south, etc.)
    rules: HashMap<u8, HashSet<u8>>, // Mapping from block type to allowed neighbors
}

impl TileRules {
    pub fn new(block_types: &[u8]) -> Self {
        let mut rules = HashMap::new();
        for &block_type in block_types {
            rules.insert(block_type, block_types.iter().cloned().collect());
        }
        TileRules { rules }
    }

    pub fn allowed_neighbors(&self, block_type: u8) -> HashSet<u8> {
        self.rules.get(&block_type)
            .cloned()
            .unwrap_or_default()
    }
}

pub fn generate_wave_function_collapse_as_octree(
    size: usize,
    block_types: &[u8],
    rules: &TileRules,
    seed: u32,
) -> OctreeNode {
    let mut rng = StdRng::seed_from_u64(seed as u64);

    // Create the root node
    let mut root = OctreeNode {
        block_types: block_types.to_vec(),
        children: None,
    };

    // Collapse the octree recursively
    collapse_octree(&mut root, size, &mut rng, rules);

    root
}

fn collapse_octree(
    node: &mut OctreeNode,
    size: usize,
    rng: &mut StdRng,
    rules: &TileRules,
) {
    if size == 1 {
        // At the smallest level, use weighted probabilities
        let block_types = node.block_types.clone();
        let weights = match size {
            0 => vec![0.8, 0.1, 0.05, 0.04, 0.01], // Bias towards lower types
            _ => vec![0.1, 0.3, 0.3, 0.2, 0.1],
        };
        
        let chosen = weighted_choose(&block_types, &weights, rng);
        node.block_types = vec![chosen];
        return;
    }

    // Subdivide the node
    let half_size = size / 2;
    let mut children: [Box<OctreeNode>; 8] = std::array::from_fn(|_| {
        Box::new(OctreeNode {
            block_types: node.block_types.clone(),
            children: None,
        })
    });

    // Collapse each child recursively and calculate constraints
    let child_constraints: Vec<(isize, isize, isize, u8)> = children
        .iter_mut()
        .enumerate()
        .map(|(i, child)| {
            let (dx, dy, dz) = (
                (i & 1) as isize * half_size as isize,
                ((i >> 1) & 1) as isize * half_size as isize,
                ((i >> 2) & 1) as isize * half_size as isize,
            );

            // Collapse the child
            collapse_octree(child, half_size, rng, rules);

            // Return constraints information
            (dx, dy, dz, child.block_types[0])
        })
        .collect();

    // Now propagate constraints separately
    for (dx, dy, dz, child_block) in child_constraints {
        propagate_constraints(node, child_block, rules, dx, dy, dz);
    }

    // Update the node's children
    node.children = Some(children);

    // Check if this node can collapse into a homogeneous block
    let first_block = node.children.as_ref().unwrap()[0].block_types[0];
    if node.children.as_ref().unwrap().iter().all(|child| child.block_types[0] == first_block) {
        node.block_types = vec![first_block];
        node.children = None; // Remove children, as the node is now homogeneous
    }
}

fn weighted_choose(types: &[u8], weights: &[f64], rng: &mut StdRng) -> u8 {
    let total_weight: f64 = weights.iter().sum();
    let rand_val = rng.gen::<f64>() * total_weight;
    
    let mut cumulative = 0.0;
    for (i, &weight) in weights.iter().enumerate() {
        cumulative += weight;
        if rand_val <= cumulative {
            return types[i];
        }
    }
    
    types[types.len() - 1]
}

// Modify propagate_constraints to take a block type instead of a mutable reference
fn propagate_constraints(
    parent: &mut OctreeNode,
    child_block: u8,
    rules: &TileRules,
    dx: isize,
    dy: isize,
    dz: isize,
) {
    parent.block_types.retain(|&block| {
        rules
            .allowed_neighbors(block)
            .contains(&child_block)
    });
}

