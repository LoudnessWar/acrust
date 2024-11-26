struct CollisionSystem {
    chunk: VoxelChunk,
    player_height: f32,
    player_width: f32,
}

//right so this makes Collision Reference Chunck so that its like more dynamic or something
impl<'a> CollisionSystem<'a> {
    fn new(chunk: &'a VoxelChunk) -> Self {
        CollisionSystem {
            chunk,
            player_height: 1.8, //this will be editable laterrrr
            player_width: 0.6,  // Player collision box width
        }
    }

    fn check_collision(&self, position: Point3<f32>) -> bool {
        // Convert world position to voxel coordinates
        let x = position.x.floor() as i32;
        let y = position.y.floor() as i32;
        let z = position.z.floor() as i32;

        // Check if the position is within chunk bounds
        if x < 0 || y < 0 || z < 0 || 
           x >= self.chunk.width() as i32 || 
           y >= self.chunk.height() as i32 || 
           z >= self.chunk.depth() as i32 {
            return false;
        }

        // Check if the block at this position is solid
        self.chunk.get_block(x as usize, y as usize, z as usize) != 0
    }

    fn resolve_collision(&mut self, current_pos: &mut Point3<f32>, proposed_pos: Point3<f32>) -> bool {
        // Simple axis-aligned collision check
        if self.check_collision(proposed_pos) {
            // If collision detected, keep the current position
            return false;
        }
        
        // Update position if no collision
        *current_pos = proposed_pos;
        true
    }
}