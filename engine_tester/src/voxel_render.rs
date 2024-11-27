use acrust::graphics::gl_wrapper::*;
use gl::types::*;
use std::mem;
use std::ptr;

use rand::prelude::*;
use std::collections::{HashMap, HashSet};

// VoxelRenderer for rendering voxels or chunks
pub struct VoxelRenderer {
    vao: Vao,
    vbo: BufferObject,
    ebo: BufferObject,
    index_count: i32,
}

impl VoxelRenderer {
    // Create a VoxelRenderer from a chunk
    pub fn from_chunk(chunk: &VoxelChunk) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0;

        let (width, height, depth) = chunk.size;

        for z in 0..depth {
            for y in 0..height {
                for x in 0..width {
                    if chunk.get_block(x, y, z) != 0 {
                        // Add cube geometry at position (x, y, z)
                        let cube_vertices = Self::generate_cube_vertices(x as f32, y as f32, z as f32);
                        let cube_indices = Self::generate_cube_indices(index_offset);

                        vertices.extend_from_slice(&cube_vertices);
                        indices.extend_from_slice(&cube_indices);

                        index_offset += 24; // 24 vertices per cube
                    }
                }
            }
        }

        let index_count = indices.len() as i32;

        // Initialize buffers
        let vao = Vao::new();
        vao.bind();

        let vbo = BufferObject::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW);
        vbo.bind();
        vbo.store_f32_data(&vertices);

        let ebo = BufferObject::new(gl::ELEMENT_ARRAY_BUFFER, gl::STATIC_DRAW);
        ebo.bind();
        ebo.store_i32_data(&indices);

        // Set vertex attributes
        let position_attribute = VertexAttribute::new(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            6 * mem::size_of::<GLfloat>() as GLsizei,
            ptr::null(),
        );
        position_attribute.enable();

        let color_attribute = VertexAttribute::new(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            6 * mem::size_of::<GLfloat>() as GLsizei,
            (3 * mem::size_of::<GLfloat>()) as *const GLvoid,
        );
        color_attribute.enable();

        VoxelRenderer { vao, vbo, ebo, index_count }
    }

    // Generate cube vertices at a specific position
    fn generate_cube_vertices(x: f32, y: f32, z: f32) -> Vec<f32> {
        vec![
            // Front face (Red)
            x - 0.5, y - 0.5, z + 0.5, 1.0, 0.0, 0.0, // Red
            x + 0.5, y - 0.5, z + 0.5, 1.0, 0.0, 0.0, // Red
            x + 0.5, y + 0.5, z + 0.5, 1.0, 0.0, 0.0, // Red
            x - 0.5, y + 0.5, z + 0.5, 0.7, 0.0, 0.0, // Red
            
            // Back face (Green)
            x - 0.5, y - 0.5, z - 0.5, 0.0, 0.7, 0.0, // Green
            x + 0.5, y - 0.5, z - 0.5, 0.0, 1.0, 0.0, // Green
            x + 0.5, y + 0.5, z - 0.5, 0.0, 1.0, 0.0, // Green
            x - 0.5, y + 0.5, z - 0.5, 0.0, 1.0, 0.0, // Green
            
            // Left Face (Blue)
            x - 0.5, y - 0.5, z - 0.5, 0.0, 0.0, 0.3, // Blue
            x - 0.5, y - 0.5, z + 0.5, 0.0, 0.0, 1.0, // Blue
            x - 0.5, y + 0.5, z + 0.5, 0.0, 0.0, 1.0, // Blue
            x - 0.5, y + 0.5, z - 0.5, 0.0, 0.0, 1.0, // Blue
            
            // Right face (Yellow)
            x + 0.5, y - 0.5, z - 0.5, 0.3, 1.0, 0.0, // Yellow
            x + 0.5, y + 0.5, z - 0.5, 1.0, 1.0, 0.0, // Yellow
            x + 0.5, y + 0.5, z + 0.5, 1.0, 0.2, 0.0, // Yellow
            x + 0.5, y - 0.5, z + 0.5, 1.0, 1.0, 0.0, // Yellow
            
            // Top face (Purple)
            x - 0.5, y + 0.5, z - 0.5, 0.5, 0.0, 1.0, // Purple
            x - 0.5, y + 0.5, z + 0.5, 0.5, 0.0, 0.3, // Purple
            x + 0.5, y + 0.5, z + 0.5, 0.3, 0.0, 1.0, // Purple
            x + 0.5, y + 0.5, z - 0.5, 0.5, 0.0, 1.0, // Purple
            
            // Bottom face (Magenta)
            x - 0.5, y - 0.5, z - 0.5, 1.0, 0.0, 1.0, // Magenta
            x + 0.5, y - 0.5, z - 0.5, 1.0, 0.0, 1.0, // Magenta
            x + 0.5, y - 0.5, z + 0.5, 1.0, 1.0, 1.0, // Magenta
            x - 0.5, y - 0.5, z + 0.5, 1.0, 0.0, 1.0, // Magenta
        ]
    }

    // Generate cube indices
    fn generate_cube_indices(offset: i32) -> Vec<i32> {
        vec![
            0 + offset, 1 + offset, 2 + offset, 0 + offset, 2 + offset, 3 + offset, // Front
            4 + offset, 5 + offset, 6 + offset, 4 + offset, 6 + offset, 7 + offset, // Back
            8 + offset, 9 + offset, 10 + offset, 8 + offset, 10 + offset, 11 + offset, // left
            12 + offset, 13 + offset, 14 + offset, 14 + offset, 15 + offset, 12 + offset, // right//ook notive how this one id different from here on out... yeah da triangles can be made like this 2 ig
            16 + offset, 17 + offset, 18 + offset, 18 + offset, 19 + offset, 16 + offset, // top
            20 + offset, 21 + offset, 22 + offset, 22 + offset, 23 + offset, 20 + offset // bottom
        ]
    }

    // Render the chunk
    pub fn render(&self) {
        unsafe {
            gl::DrawElements(gl::TRIANGLES, self.index_count, gl::UNSIGNED_INT, ptr::null());
        }
    }
}

// Tile rules define how different block types can be placed next to each other
#[derive(Clone, Debug)]
struct TileRules {
    // Mapping of block types to their allowed neighbors in each direction
    north_rules: HashMap<u8, HashSet<u8>>,
    south_rules: HashMap<u8, HashSet<u8>>,
    east_rules: HashMap<u8, HashSet<u8>>,
    west_rules: HashMap<u8, HashSet<u8>>,
    up_rules: HashMap<u8, HashSet<u8>>,
    down_rules: HashMap<u8, HashSet<u8>>,
}

pub struct VoxelChunk {
    size: (usize, usize, usize), // Dimensions of the chunk
    blocks: Vec<u8>,             // Block data
}

impl VoxelChunk {
    pub fn new(width: usize, height: usize, depth: usize) -> Self {
        let blocks = vec![0; width * height * depth]; // All blocks initially empty
        VoxelChunk {
            size: (width, height, depth),
            blocks,
        }
    }
    
    pub fn generate_wave_function_collapse(&mut self, seed: u32) {
        let mut rng = StdRng::seed_from_u64(seed as u64);
        
        // Expanded block types with more variety
        let block_types = vec![
            0,  // Air
            1,  // Grass
            2,  // Dirt
            3,  // Stone
            4,  // Bedrock
        ];
        
        // More complex and realistic tile rules
        let rules = Self::create_terrain_rules(&block_types);

        let (width, height, depth) = self.size;

        if width < 4 || height < 4 || depth < 4 {
            panic!("Chunk size too small. Minimum 4x4x4 required.");
        }

        // Initialize wave with air at the top
        let mut wave = vec![block_types.clone(); width * height * depth];
        let mut collapsed = vec![false; width * height * depth];

        // Fill top layer with air
        for x in 0..width {
            for z in 0..depth {
                for y in height - 5..height {
                    let index = x + y * width + z * width * height;
                    wave[index] = vec![0]; // Air
                    collapsed[index] = true;
                }
            }
        }

        // Create hill-like terrain
        let hill_centers = Self::generate_hill_centers(width, depth, &mut rng);
        
        for hill_center in hill_centers {
            self.generate_hill(&mut wave, &mut collapsed, &rules, 
                               hill_center.0, hill_center.1, 
                               width, height, depth, &mut rng);
        }

        // Propagate and collapse remaining cells
        for _ in 0..width * height * depth {
            if !self.iterate_wave(&mut wave, &mut collapsed, &rules, &mut rng) {
                break;
            }
        }

        // Convert wave to block data
        for x in 0..width {
            for y in 0..height {
                for z in 0..depth {
                    let index = x + y * width + z * width * height;
                    let block_type = if !collapsed[index] {
                        // If not collapsed, choose carefully
                        if wave[index].is_empty() {
                            // Fallback to a default block type
                            3 // Stone as default
                        } else {
                            wave[index][rng.gen_range(0..wave[index].len())]
                        }
                    } else {
                        wave[index][0]
                    };
                    
                    self.set_block(x, y, z, block_type);
                }
            }
        }
    }

    // Generate hill centers across the terrain
    fn generate_hill_centers(width: usize, depth: usize, rng: &mut StdRng) -> Vec<(usize, usize)> {
        // Ensure at least one hill
        let num_hills = rng.gen_range(1..=5);
        (0..num_hills)
            .map(|_| (
                rng.gen_range(0..width.max(1)), 
                rng.gen_range(0..depth.max(1))
            ))
            .collect()
    }

    // Generate a hill structure
    fn generate_hill(&mut self, 
                     wave: &mut Vec<Vec<u8>>, 
                     collapsed: &mut Vec<bool>, 
                     rules: &TileRules,
                     center_x: usize, 
                     center_z: usize, 
                     width: usize, 
                     height: usize, 
                     depth: usize, 
                     rng: &mut StdRng) {
        let hill_height = rng.gen_range(3..7);
        let hill_radius = rng.gen_range(3..6);

        for y in 0..hill_height {
            for x in center_x.saturating_sub(hill_radius)..=center_x.min(width - 1) + hill_radius {
                for z in center_z.saturating_sub(hill_radius)..=center_z.min(depth - 1) + hill_radius {
                    // Calculate distance from hill center
                    let dx = x as i32 - center_x as i32;
                    let dz = z as i32 - center_z as i32;
                    let distance = (dx * dx + dz * dz) as f32;

                    // Create circular hill shape
                    if distance <= (hill_radius * hill_radius) as f32 {
                        let index = x + (height - y - 1) * width + z * width * height;
                        
                        // Determine block type based on height
                        let block_type = match y {
                            0 => 1, // Grass on top
                            1 => 2, // Dirt below grass
                            _ => 3, // Stone deeper down
                        };

                        if index < wave.len() {
                            wave[index] = vec![block_type];
                            collapsed[index] = true;
                        }
                    }
                }
            }
        }
    }

    // Create more realistic terrain rules
    fn create_terrain_rules(block_types: &[u8]) -> TileRules {
        TileRules {
            // Air can only be at the top, blocks below air
            north_rules: Self::create_directional_rules(block_types, true),
            south_rules: Self::create_directional_rules(block_types, true),
            east_rules: Self::create_directional_rules(block_types, true),
            west_rules: Self::create_directional_rules(block_types, true),
            // Vertical rules are more strict
            up_rules: Self::create_vertical_rules(block_types),
            down_rules: Self::create_vertical_rules(block_types),
        }
    }

    // Create rules for horizontal directions
    fn create_directional_rules(block_types: &[u8], is_horizontal: bool) -> HashMap<u8, HashSet<u8>> {
        let mut rules = HashMap::new();
        for &block_type in block_types {
            let allowed = match block_type {
                0 => HashSet::new(), // Air can't have horizontal neighbors
                1 => [0, 1, 2].iter().cloned().collect(), // Grass can have air or other blocks
                2 => [0, 1, 2, 3].iter().cloned().collect(), // Dirt more flexible
                3 => [2, 3, 4].iter().cloned().collect(), // Stone typically with dirt or bedrock
                4 => [3, 4].iter().cloned().collect(), // Bedrock deep down
                _ => block_types.iter().cloned().collect(),
            };
            rules.insert(block_type, allowed);
        }
        rules
    }

    // Create rules for vertical placement
    fn create_vertical_rules(block_types: &[u8]) -> HashMap<u8, HashSet<u8>> {
        let mut rules = HashMap::new();
        for &block_type in block_types {
            let allowed = match block_type {
                0 => [1, 2, 3].iter().cloned().collect(), // Air above blocks
                1 => [2].iter().cloned().collect(), // Grass only on dirt
                2 => [3, 4].iter().cloned().collect(), // Dirt on stone or bedrock
                3 => [4].iter().cloned().collect(), // Stone on bedrock
                4 => HashSet::new(), // Bedrock at the bottom
                _ => block_types.iter().cloned().collect(),
            };
            rules.insert(block_type, allowed);
        }
        rules
    }

    // Collapse a specific point in the wave
    fn collapse_point(&mut self, 
                      wave: &mut Vec<Vec<u8>>, 
                      collapsed: &mut Vec<bool>, 
                      rules: &TileRules,
                      x: usize, y: usize, z: usize, 
                      rng: &mut StdRng) -> bool {
        let (width, height, depth) = self.size;
        let index = x + y * width + z * width * height;

        // If already collapsed, return
        if collapsed[index] {
            return false;
        }

        // Choose a block type randomly from possible states
        if !wave[index].is_empty() {
            let chosen_index = rng.gen_range(0..wave[index].len());
            let chosen_block = wave[index][chosen_index];
            
            // Keep only the chosen block
            wave[index] = vec![chosen_block];
            collapsed[index] = true;

            // Propagate constraints to neighbors
            self.propagate_constraints(wave, collapsed, rules, x, y, z);

            true
        } else {
            false
        }
    }

    // Propagate constraints to neighboring cells
    fn propagate_constraints(&mut self, 
                              wave: &mut Vec<Vec<u8>>, 
                              collapsed: &mut Vec<bool>, 
                              rules: &TileRules,
                              x: usize, y: usize, z: usize) {
        let (width, height, depth) = self.size;
        
        // Check and update each neighbor
        let neighbors = [
            (x.wrapping_sub(1), y, z, &rules.east_rules),   // West
            (x + 1, y, z, &rules.west_rules),               // East
            (x, y.wrapping_sub(1), z, &rules.up_rules),     // Down
            (x, y + 1, z, &rules.down_rules),               // Up
            (x, y, z.wrapping_sub(1), &rules.south_rules),  // North
            (x, y, z + 1, &rules.north_rules),              // South
        ];

        for (nx, ny, nz, neighbor_rules) in neighbors.iter() {
            // Check if neighbor is within bounds
            if *nx < width && *ny < height && *nz < depth {
                let neighbor_index = nx + ny * width + nz * width * height;
                
                // Skip if already collapsed
                if collapsed[neighbor_index] {
                    continue;
                }

                // Filter possible states based on current cell's state
                let current_block = wave[x + y * width + z * width * height][0];
                wave[neighbor_index].retain(|&block| 
                    neighbor_rules.get(&block)
                        .map(|allowed| allowed.contains(&current_block))
                        .unwrap_or(false)
                );
            }
        }
    }

    // Iterate through the wave and attempt to collapse points
    fn iterate_wave(&mut self, 
                    wave: &mut Vec<Vec<u8>>, 
                    collapsed: &mut Vec<bool>, 
                    rules: &TileRules, 
                    rng: &mut StdRng) -> bool {
        let (width, height, depth) = self.size;
        
        // Find the cell with the lowest entropy (least possible states)
        let mut min_entropy_index = None;
        let mut min_entropy = usize::MAX;

        for x in 0..width {
            for y in 0..height {
                for z in 0..depth {
                    let index = x + y * width + z * width * height;
                    
                    if !collapsed[index] && !wave[index].is_empty() {
                        let entropy = wave[index].len();
                        if entropy > 1 && entropy < min_entropy {
                            min_entropy = entropy;
                            min_entropy_index = Some((x, y, z));
                        }
                    }
                }
            }
        }

        // Collapse the point with lowest entropy
        if let Some((x, y, z)) = min_entropy_index {
            self.collapse_point(wave, collapsed, rules, x, y, z, rng);
            true
        } else {
            false
        }
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block_type: u8) {
        let (width, height, _) = self.size;
        self.blocks[x + y * width + z * width * height] = block_type;
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> u8 {
        let (width, height, _) = self.size;
        self.blocks[x + y * width + z * width * height]
    }
}