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

    fn update_renderer(&mut self) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut offset = 0;

        for (octree, pos) in &self.octrees {
            self.generate_geometry(octree, pos.0, pos.1, pos.2, &mut vertices, &mut indices, &mut offset);
        }

        self.renderer = Some(VoxelRenderer::new(&vertices, &indices));
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
