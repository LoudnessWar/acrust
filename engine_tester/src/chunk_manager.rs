use crate::octo::OctreeNode;
use crate::voxel_render::VoxelRenderer;

pub struct ChunkManager {
    octrees: Vec<(OctreeNode, (f32, f32, f32))>,
    renderer: Option<VoxelRenderer>,
}

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            octrees: Vec::new(),
            renderer: None,
        }
    }

    pub fn add_octree(&mut self, octree: OctreeNode, position: (f32, f32, f32)) {
        self.octrees.push((octree, position));
        self.update_renderer();
    }

    //better then hashmap you already know it
    //but ig should this be a reference or not is the only question. I think maybe not, but then when
    //later on I want to edit chunks it might be a hassle and add a lot of over head. I mean if I have to constanly
    //search a hash map in every instance of player interaction that add a whole n bro
    pub fn add_octrees(&mut self, octrees: Vec<(OctreeNode, (f32, f32, f32))>) {
        for (chunk, position) in octrees{
            self.add_octree(chunk, position);
        }
    }
    // pub fn multiple_octree_gen(){
    //     for dx in -radius..radius{
            
    //     }
    // }


    //yeah so the way i render these is dodo fart but like that not the important part ok bro
    fn update_renderer(&mut self) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut offset = 0;

        let mut count = 0;

        for (octree, pos) in &self.octrees {
            count += 1;
            self.generate_geometry(octree, pos.0, pos.1, pos.2, &mut vertices, &mut indices, &mut offset);
        }

        println!("Voxel Count: {}", count);

        self.renderer = Some(VoxelRenderer::new(&vertices, &indices));//ok so maybe this needs not a new voxelrenderer for each one? idk could run in parr
    }

    fn generate_geometry(
        &self,
        octree: &OctreeNode,
        x: f32,
        y: f32,
        z: f32,
        vertices: &mut Vec<f32>,
        indices: &mut Vec<i32>,
        offset: &mut i32,
    ) {
        if let Some(block_type) = octree.block_type() {
            if block_type != 0 {
                let color = VoxelRenderer::get_block_color(block_type);
                let (cube_vertices, cube_indices) = VoxelRenderer::generate_cube(x, y, z, color);

                vertices.extend(cube_vertices);
                indices.extend(cube_indices.iter().map(|&i| i + *offset));
                *offset += 24; // 24 vertices per cube
            }
        } else if let Some(children) = octree.children() {
            let size = 1.0; // Adjust as necessary
            for (i, child) in children.iter().enumerate() {
                let (dx, dy, dz) = (
                    (i & 1) as f32 * size,
                    ((i >> 1) & 1) as f32 * size,
                    ((i >> 2) & 1) as f32 * size,
                );
                self.generate_geometry(child, x + dx, y + dy, z + dz, vertices, indices, offset);
            }
        }
    }

    pub fn render_all(&self) {
        if let Some(renderer) = &self.renderer {
            renderer.render();
        }
    }
}
