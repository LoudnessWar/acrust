use core::f32;
use std::collections::HashMap;
use std::f64::consts::E;
use std::mem;
use std::ptr;

use gl::types::GLfloat;
use gl::types::GLvoid;

use crate::graphics::gl_wrapper::Vao;
use crate::graphics::gl_wrapper::VertexAttribute;
use crate::graphics::gl_wrapper::BufferObject;
use crate::graphics::gl_wrapper::ShaderProgram;

//ok so the mesh right should have the normals?
//yeaaahhhh
pub struct Mesh {
    vao: Vao,
    vbo: BufferObject,
    ebo: BufferObject,
    index_count: i32,
    normals_buffer: Option<BufferObject>,
    normals: Option<Vec<f32>>,
}

//should all meshes use the same VAO... PROBABLY BRO, especially if all the attributes they hold are the same so yes that means we should make a mesh manager TODO so wholesum im so excited to do that
impl Mesh {
    pub fn new(vertices: &[f32], indices: &[i32]) -> Self {
        let vao = Vao::new();
        vao.bind();//

        //verticies buffer object
        let vbo = BufferObject::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW);
        vbo.bind();
        vbo.store_f32_data(vertices);

        //like the incicides buffer object
        let ebo = BufferObject::new(gl::ELEMENT_ARRAY_BUFFER, gl::STATIC_DRAW);
        ebo.bind();
        ebo.store_i32_data(indices);//why is this not u 32?... maybe useful in some weird situation

        // Set vertex attributes

        //this one is like position
        VertexAttribute::new(0, 3, gl::FLOAT, gl::FALSE, 6 * mem::size_of::<f32>() as i32, ptr::null()).enable();//this is like the important one

        //this is for color... now, a lot of things dont do color like this and use textures so its kinda ... useless ig it can also be for normals
        //normals too maybe idk

        //yeah so this is for colors or normals it just depends on the shader... now the thing is. Like thats kinda confusing to have but like lowkey I get it fr
        VertexAttribute::new(1, 3, gl::FLOAT, gl::FALSE, 6 * mem::size_of::<GLfloat>() as i32, (3 * mem::size_of::<GLfloat>()) as *const GLvoid).enable();

        vao.unbind();//just to be safe added unbind

            Self {
            vao,
            vbo,
            ebo,
            index_count: indices.len() as i32,
            normals_buffer: None,
            normals: None,
        }
    }


    //TODO need some mesh optimization for duplicate models
    //especiialy when we calculate the normals by hand like this
    //basically... we dont want each mesh to have its own VAO's and VBO and EBO IFFF they are the same model. We can do this
    //and if it is done like this we will only need to get the normals once for that whole model
    pub fn new_normals(vertices: &mut [f32], indices: &[i32]) -> Self {
        let vao = Vao::new();
        vao.bind();

        //verticies buffer object
        let vbo = BufferObject::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW);
        vbo.bind();
        vbo.store_f32_data(vertices);

        //like the incicides buffer object
        let ebo = BufferObject::new(gl::ELEMENT_ARRAY_BUFFER, gl::STATIC_DRAW);
        ebo.bind();
        ebo.store_i32_data(indices);//why is this not u 32?... maybe useful in some weird situation

        // After loading indices, add this check:
        let max_index = *indices.iter().max().unwrap() as usize;
        if max_index >= vertices.len() {
            panic!("Index out of bounds: max_index={}, vertex_count={}", max_index, vertices.len());
        }
        // Set vertex attributes

        //this one is like position
        // why 6 why not 3... isn't it groups of 3?
        //I forget maybe it wasn't ie color???
        VertexAttribute::new(0, 3, gl::FLOAT, gl::FALSE, 6 * mem::size_of::<f32>() as i32, ptr::null()).enable();//this is like the important one

        //this is for color... now, a lot of things dont do color like this and use textures so its kinda ... useless ig it can also be for normals
        //normals too maybe idk
        VertexAttribute::new(1, 3, gl::FLOAT, gl::FALSE, 6 * mem::size_of::<GLfloat>() as i32, (3 * mem::size_of::<GLfloat>()) as *const GLvoid).enable();

        vao.unbind();//just to be safe added unbind tbh I should unbind after the thing right im like stupid for having it here? idk its like kinda working

       let mut mesh = Self {
            vao,
            vbo,
            ebo,
            index_count: indices.len() as i32,
            normals_buffer: None,
            normals: None,//im storing these for now for uuuhhh like debugging purpouses
        };

        mesh.calculate_normals(vertices, indices);
        //println!("mesh normals: {:?}", mesh.normals.as_ref().unwrap());
        // for i in 0..10 {
        //     println!("Vertex {}: Normal ({}, {}, {})", 
        //         i,
        //         mesh.normals.as_ref().unwrap()[i * 3],
        //         mesh.normals.as_ref().unwrap()[i * 3 + 1],
        //         mesh.normals.as_ref().unwrap()[i * 3 + 2]
        //     );
        // }
        //println!("mesh normals:");
        mesh 
    }

    pub fn new_normals_and_tangents(vertices: &mut [f32], indices: &[i32]) -> Self {
        let vao = Vao::new();
        vao.bind();

        //verticies buffer object
        let vbo = BufferObject::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW);
        vbo.bind();
        vbo.store_f32_data(vertices);

        //like the incicides buffer object
        let ebo = BufferObject::new(gl::ELEMENT_ARRAY_BUFFER, gl::STATIC_DRAW);
        ebo.bind();
        ebo.store_i32_data(indices);//why is this not u 32?... maybe useful in some weird situation

        // After loading indices, add this check:
        let max_index = *indices.iter().max().unwrap() as usize;
        if max_index >= vertices.len() {
            panic!("Index out of bounds: max_index={}, vertex_count={}", max_index, vertices.len());
        }
        // Set vertex attributes

        //this one is like position
        // why 6 why not 3... isn't it groups of 3?
        //I forget maybe it wasn't ie color???
        VertexAttribute::new(0, 3, gl::FLOAT, gl::FALSE, 6 * mem::size_of::<f32>() as i32, ptr::null()).enable();//this is like the important one

        //this is for color... now, a lot of things dont do color like this and use textures so its kinda ... useless ig it can also be for normals
        //normals too maybe idk
        VertexAttribute::new(1, 3, gl::FLOAT, gl::FALSE, 6 * mem::size_of::<GLfloat>() as i32, (3 * mem::size_of::<GLfloat>()) as *const GLvoid).enable();

        VertexAttribute::new(3, 3, gl::FLOAT, gl::FALSE, 6 * mem::size_of::<GLfloat>() as i32, (3 * mem::size_of::<GLfloat>()) as *const GLvoid).enable();
        
        VertexAttribute::new(4, 3, gl::FLOAT, gl::FALSE, 6 * mem::size_of::<GLfloat>() as i32, (3 * mem::size_of::<GLfloat>()) as *const GLvoid).enable();
        vao.unbind();//just to be safe added unbind tbh I should unbind after the thing right im like stupid for having it here? idk its like kinda working

       let mut mesh = Self {
            vao,
            vbo,
            ebo,
            index_count: indices.len() as i32,
            normals_buffer: None,
            normals: None,//im storing these for now for uuuhhh like debugging purpouses
        };

        mesh.calculate_normals_and_tangents(vertices, indices);

        mesh 
    }

    pub fn get_vao(&self) -> &Vao{
        &self.vao
    }

    pub fn get_index_count(&self) -> i32{//is it better to have pointer here or just like clone
        self.index_count.clone()
    }

    //this is just like a generic basic render like thing but like you need to apply textures first so thats
    //why like I will probably add a render trait to model
    pub fn draw(&self) {
        self.vao.bind();
        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                self.index_count,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }
    }

    pub fn calculate_normals(&mut self, vertices: &mut [f32], indices: &[i32]) {
        let stride = 6; // Position (3) + Normal (3)
        let normal_offset = 3; // Normal starts after position
        
        let vertex_count = vertices.len() / stride;
        let triangle_count = indices.len() / 3;
        
        //unsafe { gl::Disable(gl::CULL_FACE); }
    
        // 1. Create position buffer (xyz only)
        let positions_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::STATIC_DRAW);
        positions_buffer.bind();
        let mut positions = Vec::with_capacity(vertex_count * 3);
        for i in 0..vertex_count {
            positions.extend_from_slice(&[
                vertices[i * stride],
                vertices[i * stride + 1],
                vertices[i * stride + 2]
            ]);
        }
        positions_buffer.store_f32_data(&positions);
    
        // 2. Create face normals buffer
        let face_normals_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_DRAW);
        face_normals_buffer.bind();
        face_normals_buffer.store_f32_data(&vec![0.0; triangle_count * 3]);
    
        // 3. Create vertex normals buffer
        let vertex_normals_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_DRAW);
        vertex_normals_buffer.bind();
        vertex_normals_buffer.store_f32_data(&vec![0.0; vertex_count * 3]);
    
        // 4. Create vertex counts buffer (with atomic support)
        let vertex_counts_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_COPY);
        vertex_counts_buffer.bind();
        vertex_counts_buffer.store_i32_data(&vec![0; vertex_count]);
    
        // Set up compute shader
        let mut comp_shader = ShaderProgram::new_compute("shaders/normals.comp");
        comp_shader.bind();
        comp_shader.create_uniforms(vec!["vertex_count", "index_count", "pass"]);
        comp_shader.create_uniform("smoothingFactor");
        comp_shader.set_uniform1i("vertex_count", &(vertex_count as i32));
        comp_shader.set_uniform1i("index_count", &(indices.len() as i32));
        comp_shader.set_uniform1f("smoothingFactor", 0.5);
    
        let work_group_size = 256;
        let triangle_work_groups = (triangle_count + work_group_size - 1) / work_group_size;
        let vertex_work_groups = (vertex_count + work_group_size - 1) / work_group_size;
    
        // Pass 1: Calculate face normals
        unsafe {
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, positions_buffer.get_id());
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, self.ebo.get_id());
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, face_normals_buffer.get_id());
            comp_shader.set_uniform1i("pass", &1);
        }
        comp_shader.dispatch_compute(triangle_work_groups as u32, 1, 1);
        unsafe { gl::MemoryBarrier(gl::ALL_BARRIER_BITS); }
    
        // Pass 2: Initialize vertex buffers
        unsafe {
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 3, vertex_normals_buffer.get_id());
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 4, vertex_counts_buffer.get_id());
            comp_shader.set_uniform1i("pass", &2);
        }
        comp_shader.dispatch_compute(vertex_work_groups as u32, 1, 1);
        unsafe { gl::MemoryBarrier(gl::ALL_BARRIER_BITS); }
    
        // Pass 3: Accumulate normals
        unsafe {
            comp_shader.set_uniform1i("pass", &3);
        }
        comp_shader.dispatch_compute(triangle_work_groups as u32, 1, 1);
        unsafe { gl::MemoryBarrier(gl::ALL_BARRIER_BITS); }
    
        // Pass 4: Normalize vertex normals
        unsafe {
            comp_shader.set_uniform1i("pass", &4);
        }
        comp_shader.dispatch_compute(vertex_work_groups as u32, 1, 1);
        unsafe { gl::MemoryBarrier(gl::ALL_BARRIER_BITS); }
    
        // Read back results
        let mut calculated_normals = vec![0.0; vertex_count * 3];
        vertex_normals_buffer.bind();
        unsafe {
            gl::GetBufferSubData(
                gl::SHADER_STORAGE_BUFFER,
                0,
                (vertex_count * 3 * std::mem::size_of::<f32>()) as isize,
                calculated_normals.as_mut_ptr() as *mut _
            );
        }
    
        // Update vertex data with new normals
        for i in 0..vertex_count {
            let base = i * stride + normal_offset;
            vertices[base..base+3].copy_from_slice(&calculated_normals[i*3..i*3+3]);
        }
    
        // Debug output
        println!("First 10 normals:");
        for i in 0..10.min(vertex_count) {
            println!("Vertex {}: ({:.3}, {:.3}, {:.3})", 
                i, 
                calculated_normals[i*3], 
                calculated_normals[i*3+1], 
                calculated_normals[i*3+2]
            );
        }
    
        // Update VBO
        self.vbo.bind();
        self.vbo.store_f32_data(vertices);
    }

    pub fn new_normals_test(vertices: &mut [f32], indices: &[i32]) -> Self {
        let vao = Vao::new();
        vao.bind();

        //verticies buffer object
        let vbo = BufferObject::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW);
        vbo.bind();
        vbo.store_f32_data(vertices);

        //like the incicides buffer object
        let ebo = BufferObject::new(gl::ELEMENT_ARRAY_BUFFER, gl::STATIC_DRAW);
        ebo.bind();
        ebo.store_i32_data(indices);//why is this not u 32?... maybe useful in some weird situation

        // After loading indices, add this check:
        let max_index = *indices.iter().max().unwrap() as usize;
        if max_index >= vertices.len() {
            panic!("Index out of bounds: max_index={}, vertex_count={}", max_index, vertices.len());
        }
        // Set vertex attributes

        //this one is like position
        // why 6 why not 3... isn't it groups of 3?
        //I forget maybe it wasn't ie color???
        VertexAttribute::new(0, 3, gl::FLOAT, gl::FALSE, 6 * mem::size_of::<f32>() as i32, ptr::null()).enable();//this is like the important one

        //this is for color... now, a lot of things dont do color like this and use textures so its kinda ... useless ig it can also be for normals
        //normals too maybe idk
        VertexAttribute::new(1, 3, gl::FLOAT, gl::FALSE, 6 * mem::size_of::<GLfloat>() as i32, (3 * mem::size_of::<GLfloat>()) as *const GLvoid).enable();

        vao.unbind();//just to be safe added unbind tbh I should unbind after the thing right im like stupid for having it here? idk its like kinda working

       let mut mesh = Self {
            vao,
            vbo,
            ebo,
            index_count: indices.len() as i32,
            normals_buffer: None,
            normals: None,//im storing these for now for uuuhhh like debugging purpouses
        };

        mesh.calculate_normals2(vertices, indices);
        //println!("mesh normals: {:?}", mesh.normals.as_ref().unwrap());
        // for i in 0..10 {
        //     println!("Vertex {}: Normal ({}, {}, {})", 
        //         i,
        //         mesh.normals.as_ref().unwrap()[i * 3],
        //         mesh.normals.as_ref().unwrap()[i * 3 + 1],
        //         mesh.normals.as_ref().unwrap()[i * 3 + 2]
        //     );
        // }
        //println!("mesh normals:");
        mesh 
    }

    pub fn calculate_normals2(&mut self, vertices: &mut [f32], indices: &[i32]) {
        let stride = 6; // Position (3) + Normal (3)
        let normal_offset = 3; // Normal starts after position
        
        let vertex_count = vertices.len() / stride;
        let triangle_count = indices.len() / 3;
        
        // 1. Create position buffer (xyz only)
        let positions_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::STATIC_DRAW);
        positions_buffer.bind();
        let mut positions = Vec::with_capacity(vertex_count * 3);
        for i in 0..vertex_count {
            positions.extend_from_slice(&[
                vertices[i * stride],
                vertices[i * stride + 1],
                vertices[i * stride + 2]
            ]);
        }
        positions_buffer.store_f32_data(&positions);
    
        // 2. Create face normals buffer
        let face_normals_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_DRAW);
        face_normals_buffer.bind();
        face_normals_buffer.store_f32_data(&vec![0.0; triangle_count * 3]);
    
        // 3. Create vertex normals buffer
        let vertex_normals_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_DRAW);
        vertex_normals_buffer.bind();
        vertex_normals_buffer.store_f32_data(&vec![0.0; vertex_count * 3]);
    
        // 4. Create vertex counts buffer (with atomic support)
        // Note: For flat shading we don't actually need this, but keeping for compatibility
        let vertex_counts_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_COPY);
        vertex_counts_buffer.bind();
        vertex_counts_buffer.store_i32_data(&vec![0; vertex_count]);
    
        // Set up compute shader
        let mut comp_shader = ShaderProgram::new_compute("shaders/basic_normals.comp");
        comp_shader.bind();
        comp_shader.create_uniforms(vec!["vertex_count", "index_count", "pass"]);
        comp_shader.set_uniform1i("vertex_count", &(vertex_count as i32));
        comp_shader.set_uniform1i("index_count", &(indices.len() as i32));
    
        let work_group_size = 256;
        let triangle_work_groups = (triangle_count + work_group_size - 1) / work_group_size;
        let vertex_work_groups = (vertex_count + work_group_size - 1) / work_group_size;
    
        // Pass 1: Calculate face normals and directly assign to vertices (flat shading)
        unsafe {
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, positions_buffer.get_id());
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, self.ebo.get_id());
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, face_normals_buffer.get_id());
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 3, vertex_normals_buffer.get_id());
            comp_shader.set_uniform1i("pass", &1);
        }
        comp_shader.dispatch_compute(triangle_work_groups as u32, 1, 1);
        unsafe { gl::MemoryBarrier(gl::ALL_BARRIER_BITS); }
    
        // Since we're doing flat shading, we can skip passes 2, 3, and 4
        // We just need pass 1 to calculate face normals and assign them to vertices
        
        // Read back results from vertex normals buffer
        let mut calculated_normals = vec![0.0; vertex_count * 3];
        vertex_normals_buffer.bind();
        unsafe {
            gl::GetBufferSubData(
                gl::SHADER_STORAGE_BUFFER,
                0,
                (vertex_count * 3 * std::mem::size_of::<f32>()) as isize,
                calculated_normals.as_mut_ptr() as *mut _
            );
        }
    
        // Update vertex data with new normals
        for i in 0..vertex_count {
            let base = i * stride + normal_offset;
            vertices[base..base+3].copy_from_slice(&calculated_normals[i*3..i*3+3]);
        }
    
        // Debug output
        println!("First 10 normals (flat shading):");
        for i in 0..10.min(vertex_count) {
            println!("Vertex {}: ({:.3}, {:.3}, {:.3})", 
                i, 
                calculated_normals[i*3], 
                calculated_normals[i*3+1], 
                calculated_normals[i*3+2]
            );
        }
    
        // Update VBO
        self.vbo.bind();
        self.vbo.store_f32_data(vertices);
    }

    pub fn calculate_normals_and_tangents(&mut self, vertices: &mut [f32], indices: &[i32]) {
        let stride = 6; // Position (3) + Normal (3)
        let normal_offset = 3;
        
        let vertex_count = vertices.len() / stride; // Assuming 6 floats per vertex (position + normal)
        let triangle_count = indices.len() / 3;
    
        // Create buffers (reusing your existing buffer creation code)
        let positions_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::STATIC_DRAW);
        positions_buffer.bind();
        let mut positions = Vec::with_capacity(vertex_count * 3);
        for i in 0..vertex_count {
            positions.extend_from_slice(&[
                vertices[i * stride],
                vertices[i * stride + 1],
                vertices[i * stride + 2]
            ]);
        }
        positions_buffer.store_f32_data(&positions);

        let face_normals_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_DRAW);
        face_normals_buffer.bind();
        face_normals_buffer.store_f32_data(&vec![0.0; triangle_count * 3]);
    
        // 3. Create vertex normals buffer
        let vertex_normals_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_DRAW);
        vertex_normals_buffer.bind();
        vertex_normals_buffer.store_f32_data(&vec![0.0; vertex_count * 3]);
    
        // 4. Create vertex counts buffer (with atomic support)
        let vertex_counts_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_COPY);
        vertex_counts_buffer.bind();
        vertex_counts_buffer.store_i32_data(&vec![0; vertex_count]);
    
        // Create new buffers for tangents/bitangents
        let tangents_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_DRAW);
        let bitangents_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_DRAW);
    
        // Set up compute shader
        let mut comp_shader = ShaderProgram::new_compute("shaderfp/normals.comp");
        comp_shader.bind();
        comp_shader.create_uniforms(vec!["vertex_count", "index_count", "pass"]);
        comp_shader.create_uniform("smoothingFactor");
        comp_shader.set_uniform1i("vertex_count", &(vertex_count as i32));
        comp_shader.set_uniform1i("index_count", &(indices.len() as i32));
        comp_shader.set_uniform1f("smoothingFactor", 0.5);

        let work_group_size = 256;
        let triangle_work_groups = (triangle_count + work_group_size - 1) / work_group_size;
        let vertex_work_groups = (vertex_count + work_group_size - 1) / work_group_size;
        
        // Bind all buffers (add the new ones)
        unsafe {
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, positions_buffer.get_id());
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, self.ebo.get_id());
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, face_normals_buffer.get_id());
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 3, vertex_normals_buffer.get_id());
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 4, vertex_counts_buffer.get_id());
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 5, tangents_buffer.get_id());
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 6, bitangents_buffer.get_id());
        }
    
        comp_shader.dispatch_compute(triangle_work_groups as u32, 1, 1);
        unsafe { gl::MemoryBarrier(gl::ALL_BARRIER_BITS); }
    
        // Pass 2: Initialize vertex buffers
        unsafe {
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 3, vertex_normals_buffer.get_id());
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 4, vertex_counts_buffer.get_id());
            comp_shader.set_uniform1i("pass", &2);
        }
        comp_shader.dispatch_compute(vertex_work_groups as u32, 1, 1);
        unsafe { gl::MemoryBarrier(gl::ALL_BARRIER_BITS); }
    
        // Pass 3: Accumulate normals
        unsafe {
            comp_shader.set_uniform1i("pass", &3);
        }
        comp_shader.dispatch_compute(triangle_work_groups as u32, 1, 1);
        unsafe { gl::MemoryBarrier(gl::ALL_BARRIER_BITS); }
    
        // Pass 4: Normalize vertex normals
        unsafe {
            comp_shader.set_uniform1i("pass", &4);
        }
        comp_shader.dispatch_compute(vertex_work_groups as u32, 1, 1);
        unsafe { gl::MemoryBarrier(gl::ALL_BARRIER_BITS); }
    
        // After compute shader completes, read back the results
        let mut new_vertices = Vec::with_capacity(vertex_count * 12); // 12 floats per vertex
        
        // Read normals, tangents, bitangents from buffers
        let mut normals = vec![0.0; vertex_count * 3];
        let mut tangents = vec![0.0; vertex_count * 3];
        let mut bitangents = vec![0.0; vertex_count * 3];
        
        vertex_normals_buffer.bind();
        unsafe {
            gl::GetBufferSubData(
                gl::SHADER_STORAGE_BUFFER,
                0,
                (vertex_count * 3 * std::mem::size_of::<f32>()) as isize,
                normals.as_mut_ptr() as *mut _
            );
        }
        
        tangents_buffer.bind();
        unsafe {
            gl::GetBufferSubData(
                gl::SHADER_STORAGE_BUFFER,
                0,
                (vertex_count * 3 * std::mem::size_of::<f32>()) as isize,
                tangents.as_mut_ptr() as *mut _
            );
        }
    
        // Create new interleaved vertex data
        for i in 0..vertex_count {
            // Position (3 floats)
            new_vertices.extend_from_slice(&vertices[i*6..i*6+3]);
            
            // Normal (3 floats)
            new_vertices.extend_from_slice(&normals[i*3..i*3+3]);
            
            // Tangent (3 floats)
            new_vertices.extend_from_slice(&tangents[i*3..i*3+3]);
            
            // Bitangent (3 floats) - could be calculated in shader instead
            new_vertices.extend_from_slice(&bitangents[i*3..i*3+3]);
        }

        println!("First 10 normals:");
        for i in 0..10.min(vertex_count) {
            println!("Vertex {}: ({:.3}, {:.3}, {:.3})", 
                i, 
                new_vertices[i*3], 
                new_vertices[i*3+1], 
                new_vertices[i*3+2]
            );
        }
    
        // Update VBO with new vertex format
        self.vbo.bind();
        self.vbo.store_f32_data(&new_vertices);
    
        // Update VAO attributes
        // self.vao.bind();
        // let float_size = std::mem::size_of::<f32>() as i32;
    
        // // Position (location = 0)
        // VertexAttribute::new(0, 3, gl::FLOAT, gl::FALSE, 12 * float_size, ptr::null())
        //     .enable();
    
        // // Normal (location = 1)
        // VertexAttribute::new(1, 3, gl::FLOAT, gl::FALSE, 12 * float_size, (3 * float_size) as *const _)
        //     .enable();
    
        // // Tangent (location = 2)
        // VertexAttribute::new(2, 3, gl::FLOAT, gl::FALSE, 12 * float_size, (6 * float_size) as *const _)
        //     .enable();
    
        // // Bitangent (location = 3) - Optional, can be calculated in shader
        // VertexAttribute::new(3, 3, gl::FLOAT, gl::FALSE, 12 * float_size, (9 * float_size) as *const _)
        //     .enable();
    
        self.vao.unbind();
    }

    /////////////EXZZZZ

    pub fn ez_calculate_normals(&mut self, vertices: &[f32], indices: &[i32]) {
        
        //bufffer base for ssbo buffers that is not the title of a smash game btw ssb Omega or Odyssy 
        unsafe {
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, self.vbo.get_id());
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, self.ebo.get_id());
        }
    
        let vertex_count = vertices.len() / 6;
    
        let mut cleared_data = vertices.to_vec();
        for i in 0..vertex_count {
            cleared_data[i * 6 + 3] = 0.0;
            cleared_data[i * 6 + 4] = 0.0;
            cleared_data[i * 6 + 5] = 0.0;
        }
        self.vbo.bind();
        self.vbo.store_f32_data(&cleared_data);
    
        // lol look at the TODO this should not be here. Because like I dsont need to recompline
        //ect ect ect for the compute shader everytime a new model is made. MAYBE I just
        //need a like mesh Manger I think
        //I propose/hypothisized over the creation of one previously
        let mut comp_shader = ShaderProgram::new_compute("shaders/eznorms.comp");
        comp_shader.bind();
        comp_shader.create_uniforms(vec!["vertex_count", "index_count"]);
        comp_shader.set_uniform1i("vertex_count", &(vertex_count as i32));
        comp_shader.set_uniform1i("index_count", &(indices.len() as i32));
    
        let work_group_size = 256;
        let num_work_groups = (vertex_count + work_group_size - 1) / work_group_size;
        comp_shader.dispatch_compute(num_work_groups as u32, 1, 1);
    
        // BOOOORIng dont crash pal
        unsafe {
            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
        }
    
        // getting it back with MapBufferRange like tbh not neededanymore but imma keep it
        //TODO maybe remove later
        unsafe {
            let size = (vertex_count * 6 * std::mem::size_of::<f32>()) as isize;
            let ptr = gl::MapBufferRange(
                gl::SHADER_STORAGE_BUFFER,
                0,
                size,
                gl::MAP_READ_BIT
            ) as *const f32;
    
            if !ptr.is_null() {
                let data = std::slice::from_raw_parts(ptr, vertex_count * 6);
                let mut normals = Vec::with_capacity(vertex_count * 3);
                for i in 0..vertex_count {
                    normals.push(data[i * 6 + 3]);
                    normals.push(data[i * 6 + 4]);
                    normals.push(data[i * 6 + 5]);
                }
                self.normals = Some(normals);
                gl::UnmapBuffer(gl::SHADER_STORAGE_BUFFER);
            }
        }
    }
}

fn check_gl_error(label: &str) {
    let err = unsafe { gl::GetError() };
    if err != gl::NO_ERROR {
        panic!("GL Error after {}: 0x{:X}", label, err);
    }
}


//i want to add a 3d object trait here with a basic render and like basic funciton ect ect

//ok so i did what i said above but in objload which uuuh might change its location yall

//pub fn calculate_tangent_basis(&mut self, vertices: &mut [f32], indices: &[i32]) {
    //     let stride = 6; // Position (3) + Normal (3)
    //     let vertex_count = vertices.len() / stride;
    //     let triangle_count = indices.len() / 3;
    
    //     // Initialize tangent and bitangent arrays
    //     let mut tangents = vec![0.0; vertex_count * 3];
    //     let mut bitangents = vec![0.0; vertex_count * 3];
    //     let mut tangent_counts = vec![0; vertex_count];
    
    //     // Process each triangle
    //     for i in 0..triangle_count {
    //         let i0 = indices[i * 3] as usize;
    //         let i1 = indices[i * 3 + 1] as usize;
    //         let i2 = indices[i * 3 + 2] as usize;
    
    //         // Get vertex positions
    //         let v0 = &vertices[i0 * stride..i0 * stride + 3];
    //         let v1 = &vertices[i1 * stride..i1 * stride + 3];
    //         let v2 = &vertices[i2 * stride..i2 * stride + 3];
    
    //         // Get texture coordinates (assuming they exist)
    //         // You'll need to modify this if your vertex layout is different
    //         let uv0 = &vertices[i0 * stride + 6..i0 * stride + 8]; // Assuming texcoords are at offset 6
    //         let uv1 = &vertices[i1 * stride + 6..i1 * stride + 8];
    //         let uv2 = &vertices[i2 * stride + 6..i2 * stride + 8];
    
    //         // Calculate edges
    //         let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    //         let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    
    //         // Calculate UV differences
    //         let delta_uv1 = [uv1[0] - uv0[0], uv1[1] - uv0[1]];
    //         let delta_uv2 = [uv2[0] - uv0[0], uv2[1] - uv0[1]];
    
    //         let f = 1.0 / (delta_uv1[0] * delta_uv2[1] - delta_uv2[0] * delta_uv1[1]);
    
    //         // Calculate tangent and bitangent for this triangle
    //         let tangent = [
    //             f * (delta_uv2[1] * edge1[0] - delta_uv1[1] * edge2[0]),
    //             f * (delta_uv2[1] * edge1[1] - delta_uv1[1] * edge2[1]),
    //             f * (delta_uv2[1] * edge1[2] - delta_uv1[1] * edge2[2]),
    //         ];
    
    //         let bitangent = [
    //             f * (-delta_uv2[0] * edge1[0] + delta_uv1[0] * edge2[0]),
    //             f * (-delta_uv2[0] * edge1[1] + delta_uv1[0] * edge2[1]),
    //             f * (-delta_uv2[0] * edge1[2] + delta_uv1[0] * edge2[2]),
    //         ];
    
    //         // Accumulate tangents and bitangents
    //         for &index in &[i0, i1, i2] {
    //             tangents[index * 3] += tangent[0];
    //             tangents[index * 3 + 1] += tangent[1];
    //             tangents[index * 3 + 2] += tangent[2];
    
    //             bitangents[index * 3] += bitangent[0];
    //             bitangents[index * 3 + 1] += bitangent[1];
    //             bitangents[index * 3 + 2] += bitangent[2];
    
    //             tangent_counts[index] += 1;
    //         }
    //     }
    
    //     // Normalize and orthogonalize tangents
    //     for i in 0..vertex_count {
    //         if tangent_counts[i] > 0 {
    //             // Average the tangents
    //             let inv_count = 1.0 / tangent_counts[i] as f32;
    //             let mut tangent = [
    //                 tangents[i * 3] * inv_count,
    //                 tangents[i * 3 + 1] * inv_count,
    //                 tangents[i * 3 + 2] * inv_count
    //             ];
    
    //             // Get the normal
    //             let normal = &vertices[i * stride + 3..i * stride + 6];
    
    //             // Gram-Schmidt orthogonalize
    //             let n_dot_t = normal[0] * tangent[0] + normal[1] * tangent[1] + normal[2] * tangent[2];
    //             tangent[0] -= normal[0] * n_dot_t;
    //             tangent[1] -= normal[1] * n_dot_t;
    //             tangent[2] -= normal[2] * n_dot_t;
    
    //             // Normalize
    //             let len = (tangent[0] * tangent[0] + tangent[1] * tangent[1] + tangent[2] * tangent[2]).sqrt();
    //             if len > 0.0 {
    //                 let inv_len = 1.0 / len;
    //                 tangent[0] *= inv_len;
    //                 tangent[1] *= inv_len;
    //                 tangent[2] *= inv_len;
    //             }
    
    //             // Store back
    //             tangents[i * 3..i * 3 + 3].copy_from_slice(&tangent);
    //         }
    //     }
    
    //     // Now you need to modify your vertex buffer to include tangents and bitangents
    //     // This depends on your vertex format - you might need to:
    //     // 1. Create a new vertex buffer with additional attributes
    //     // 2. Interleave the tangent/bitangent data with existing data
    //     // 3. Update your VAO to include the new attributes
    
    //     // For debugging:
    //     println!("First 10 tangents:");
    //     for i in 0..10.min(vertex_count) {
    //         println!("Vertex {}: T({:.3}, {:.3}, {:.3}) B({:.3}, {:.3}, {:.3})", 
    //             i, 
    //             tangents[i*3], tangents[i*3+1], tangents[i*3+2],
    //             bitangents[i*3], bitangents[i*3+1], bitangents[i*3+2]
    //         );
    //     }

    //     for i in 0..vertex_count {
    //         let base = i * stride + normal_offset;
    //         vertices[base..base+3].copy_from_slice(&calculated_normals[i*3..i*3+3]);
    //     }
    // }

    // pub fn calculate_normals(&mut self, vertices: &[f32], indices: &[i32]) {
    //     let vertex_count = vertices.len() / 6; // Assuming 6 floats per vertex (pos + something else)
    //     let triangle_count = indices.len() / 3;
        
    //     // Create buffers for the multi-pass approach
        
    //     // 1. Face normals buffer (temporary)
    //     let face_normals_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_DRAW);
    //     face_normals_buffer.bind();
    //     let face_normal_data = vec![0.0f32; triangle_count * 3]; // 3 components per face normal
    //     face_normals_buffer.store_f32_data(&face_normal_data);
        
    //     // 2. Vertex normals buffer (output)
    //     let vertex_normals_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_DRAW);
    //     vertex_normals_buffer.bind();
    //     let vertex_normal_data = vec![0.0f32; vertex_count * 3]; // 3 components per vertex normal
    //     vertex_normals_buffer.store_f32_data(&vertex_normal_data);
        
    //     // 3. Vertex triangle counts buffer
    //     print!("herhe");
    //     let vertex_counts_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_DRAW);
    //     vertex_counts_buffer.bind();
    //     let vertex_count_data = vec![0i32; vertex_count]; // One int per vertex
    //     vertex_counts_buffer.store_i32_data(&vertex_count_data);
        
    //     // Set up and run compute shader
    //     let mut comp_shader = ShaderProgram::new_compute("shaders/normals.comp");
    //     comp_shader.bind();

    //     comp_shader.create_uniforms(vec!["vertex_count", "index_count", "pass"]);
        
    //     // Common uniform setup
    //     comp_shader.set_uniform1i("vertex_count", &(vertex_count as i32));
    //     comp_shader.set_uniform1i("index_count", &(indices.len() as i32));
        
    //     // Calculate work group sizes
    //     let work_group_size = 256; // Typical compute shader workgroup size
    //     let triangle_work_groups = (triangle_count + work_group_size - 1) / work_group_size;
    //     let vertex_work_groups = (vertex_count + work_group_size - 1) / work_group_size;
        
    //     // Pass 1: Calculate face normals
    //     print!("Calculate face normals");
    //     unsafe {
    //         // Bind buffers to the compute shader
    //         gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, self.vbo.get_id());
    //         gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, self.ebo.get_id());
    //         gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, face_normals_buffer.get_id());
    //         check_gl_error("poop 2");
    //         comp_shader.set_uniform1i("pass", &1);
    //     }
        
    //     comp_shader.dispatch_compute(triangle_work_groups as u32, 1, 1);
        
    //     // Ensure Pass 1 is complete
    //     unsafe {
    //         gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
    //     }
        
    //     // Pass 2: Initialize vertex normals and counting buffers
    //     unsafe {
    //         println!("Initialize vertex normals and counting buffers");
    //         gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 3, vertex_normals_buffer.get_id());
    //         gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 4, vertex_counts_buffer.get_id());
    //         check_gl_error("poop");
    //         comp_shader.set_uniform1i("pass", &2);
    //     }
        
    //     comp_shader.dispatch_compute(vertex_work_groups as u32, 1, 1);
        
    //     // Ensure Pass 2 is complete
    //     unsafe {
    //         gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
    //     }
        
    //     // Pass 3: Accumulate face normals to vertex normals
    //     unsafe {
    //         comp_shader.set_uniform1i("pass", &3);
    //     }
        
    //     comp_shader.dispatch_compute(triangle_work_groups as u32, 1, 1);
        
    //     // Ensure Pass 3 is complete
    //     unsafe {
    //         gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
    //     }
    //     check_gl_error("unmapped poop 5");
    //     // Pass 4: Normalize vertex normals
    //     unsafe {
    //         comp_shader.set_uniform1i("pass", &4);
    //     }
        
    //     comp_shader.dispatch_compute(vertex_work_groups as u32, 1, 1);
        
    //     // Ensure Pass 4 is complete
    //     unsafe {
    //         gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
    //     }
        
    //     check_gl_error("unmapped poop 56");
    //     println!("Map the normals buffer to read the results");
    //     // Map the normals buffer to read the results
    //     vertex_normals_buffer.bind();
    //     unsafe {
    //         let size = (vertex_count * 3 * std::mem::size_of::<f32>()) as isize;
    //         let ptr = gl::MapBufferRange(
    //             gl::SHADER_STORAGE_BUFFER, 
    //             0,
    //             size,
    //             gl::MAP_READ_BIT
    //         ) as *const f32;
    //         check_gl_error("unmapped poop 4");
    //         if !ptr.is_null() {
    //             // Copy data from GPU to CPU
    //             let normals_slice = std::slice::from_raw_parts(ptr, vertex_count * 3);
    //             self.normals = Some(normals_slice.to_vec());
                
    //             // Unmap the buffer
    //             gl::UnmapBuffer(gl::SHADER_STORAGE_BUFFER);
    //             check_gl_error("unmapped poop");
    //         }

            
    //     }
    //     check_gl_error("unmapped poop 2");
    //     // Store the normals buffer in the struct
    //     self.normals_buffer = Some(vertex_normals_buffer);
    //     check_gl_error("unmapped poop 3");
        
    //     // Clean up temporary buffers
    //     // Note: We don't need to explicitly delete face_normals_buffer and vertex_counts_buffer
    //     // as they will be automatically cleaned up when they go out of scope
        
    //     // Now update your VAO to include normals
    //     self.vao.bind();
        
    //     //check_gl_error("vnormals_buffer");
    //     if let Some(buffer) = &self.normals_buffer {
    //         buffer.bind();
            
    //         check_gl_error("poop inside");
    //         // Set up the normal attribute (assuming it's attribute 1)
    //         VertexAttribute::new(
    //             1, // attribute index for normals
    //             3, // 3 components for normals
    //             gl::FLOAT,
    //             gl::FALSE,
    //             3 * std::mem::size_of::<f32>() as i32, // stride (3 floats per normal)
    //             std::ptr::null()
    //         ).enable();
    //     }
        
    //     self.vao.unbind();
    // }
    

    // pub fn calculate_normals(&self, vertices: &[f32], indices: &[i32]) -> Option<Vec<f32>>{
    //     // Allocate space for the normals (one normal per vertex)
    //     let vertex_count = vertices.len() / 6; // Assuming 6 floats per vertex (pos + something else)
    //     let normal_data = vec![0.0f32; vertex_count * 3]; // 3 components per normal

    //     // Set up and run compute shader which like TBH should not BE HERE because then it will be remade/compiled multiple times when that does not need to happen
    //     //TODO fix dis
    //     let mut comp_shader = ShaderProgram::new_compute("shaders/normals.comp");
    //     comp_shader.bind();
    //     comp_shader.create_uniforms(vec!["vertex_count", "index_count"]);
    //     // Bind buffers to the compute shader
    //     unsafe {
    //         // Bind vertex buffer (read-only)
    //         gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, self.vbo.get_id());
            
    //         // Bind index buffer (read-only)
    //         gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, self.ebo.get_id());
            
    //         // Bind normals buffer (write)
    //         gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, normals_buffer.get_id());
            
    //         // Pass necessary uniform data to the compute shader
    //         comp_shader.set_uniform1i("vertex_count", &(vertex_count as i32));//lovely it only takes references really
    //         comp_shader.set_uniform1i("index_count", &(indices.len() as i32));
    //     }
        
    //     // Dispatch compute shader with appropriate workgroup size
    //     // Calculate work group count based on vertex count
    //     let work_group_size = 256; // Typical compute shader workgroup size
    //     let num_work_groups = (vertex_count + work_group_size - 1) / work_group_size;
    //     comp_shader.dispatch_compute(num_work_groups as u32, 1, 1);
        
    //     // Make sure the compute shader is done
    //     unsafe {
    //         gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
    //     }
        
    //     // Map the normals buffer to read the results
    //     unsafe {
    //         let size = (vertex_count * 3 * std::mem::size_of::<f32>()) as isize;
    //         let ptr = gl::MapBufferRange(
    //             gl::SHADER_STORAGE_BUFFER, 
    //             0,
    //             size,
    //             gl::MAP_READ_BIT
    //         ) as *const f32;
            
    //         if !ptr.is_null() {
    //             // Copy data from GPU to CPU
    //             let normals_slice = std::slice::from_raw_parts(ptr, vertex_count * 3);
    //             self.normals = Some(normals_slice.to_vec());
                
    //             // Unmap the buffer
    //             gl::UnmapBuffer(gl::SHADER_STORAGE_BUFFER);
    //         }
    //     }

    //     Some(normals_buffer)
        
    //     // Store the normals buffer in the struct
    //     // self.normals_buffer = Some(normals_buffer);
        
    //     // // Now update your VAO to include normals
    //     // self.vao.bind();

    //     // if let Some(buffer) = self.normals_buffer.as_mut() {
    //     //     buffer.bind();
    //     // }
        
    //     // // Set up the normal attribute (assuming it's attribute 1)
    //     // // VertexAttribute::new(
    //     // //     1, // attribute index for normals
    //     // //     3, // 3 components for normals
    //     // //     gl::FLOAT,
    //     // //     gl::FALSE,
    //     // //     3 * std::mem::size_of::<f32>() as i32, // stride (3 floats per normal)
    //     // //     std::ptr::null()
    //     // // ).enable();
        
    //     // self.vao.unbind();
    // }

    
    // pub fn calculate_normals(&mut self, vertices: &mut [f32], indices: &[i32]) {
    //     let stride = 6; // WE DONT HAVE A STRIDE FOR TEXCOORDS YET TODO CHANGE STRIDE FOR IT WHEN NEEDED
    //     let normal_offset = 3; // Position of normal.x in your vertex format
        
    //     let vertex_count = vertices.len() / stride;
    //     let triangle_count = indices.len() / 3;
        
    //     // Create buffers for the multi-pass approach
    //     unsafe {
    //         gl::Disable(gl::CULL_FACE);
    //     }
        
    //     // 1. Face normals buffer (temporary)
    //     let face_normals_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_DRAW);
    //     face_normals_buffer.bind();
    //     let face_normal_data = vec![0.0f32; triangle_count * 3]; // 3 components per face normal
    //     face_normals_buffer.store_f32_data(&face_normal_data);
        
    //     // 2. Vertex normals buffer (temporary output)
    //     let vertex_normals_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_DRAW);
    //     vertex_normals_buffer.bind();
    //     let vertex_normal_data = vec![0.0f32; vertex_count * 3]; // 3 components per vertex normal
    //     vertex_normals_buffer.store_f32_data(&vertex_normal_data);
        
    //     // 3. Vertex triangle counts buffer
    //     let vertex_counts_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_DRAW);
    //     vertex_counts_buffer.bind();
    //     let vertex_count_data = vec![0i32; vertex_count]; // One int per vertex
    //     vertex_counts_buffer.store_i32_data(&vertex_count_data);
        
    //     // Create a temporary buffer with just the positions for the compute shader
    //     let positions_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::STATIC_DRAW);
    //     positions_buffer.bind();
        
    //     // dude... what is the point of this even like I could just have a stride of 6 in the compute... like idk man
    //     let mut positions = Vec::with_capacity(vertex_count * 3);
    //     for i in 0..vertex_count {
    //         positions.push(vertices[i * stride]);     // pos.x
    //         positions.push(vertices[i * stride + 1]); // pos.y
    //         positions.push(vertices[i * stride + 2]); // pos.z
    //     }
    //     positions_buffer.store_f32_data(&positions);
    //     println!("First 16 position floats: {:?}", &positions[0..16]);
        
    //     // Set up and run compute shader
    //     let mut comp_shader = ShaderProgram::new_compute("shaders/normals.comp");
    //     comp_shader.bind();
    //     comp_shader.create_uniforms(vec!["vertex_count", "index_count", "pass"]);
        
    //     // Common uniform setup
    //     comp_shader.set_uniform1i("vertex_count", &(vertex_count as i32));
    //     comp_shader.set_uniform1i("index_count", &(indices.len() as i32));
        
    //     // Calculate work group sizes
    //     let work_group_size = 256; // Typical compute shader workgroup size
    //     let triangle_work_groups = (triangle_count + work_group_size - 1) / work_group_size;
    //     let vertex_work_groups = (vertex_count + work_group_size - 1) / work_group_size;
        
    //     // Pass 1: Calculate face normals
    //     unsafe {
    //         // Bind buffers to the compute shader
    //         gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, positions_buffer.get_id());
    //         gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, self.ebo.get_id());
    //         gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, face_normals_buffer.get_id());
            
    //         comp_shader.set_uniform1i("pass", &1);
    //     }
        
    //     comp_shader.dispatch_compute(triangle_work_groups as u32, 1, 1);
        
    //     // Ensure Pass 1 is complete
    //     unsafe {
    //         gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
    //     }
    //     unsafe {
    //         gl::MemoryBarrier(gl::ALL_BARRIER_BITS);
    //     }
        
    //     // Pass 2: Initialize vertex normals and counting buffers
    //     unsafe {
    //         gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 3, vertex_normals_buffer.get_id());
    //         gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 4, vertex_counts_buffer.get_id());
            
    //         comp_shader.set_uniform1i("pass", &2);
    //     }
        
    //     comp_shader.dispatch_compute(vertex_work_groups as u32, 1, 1);
        
    //     // Ensure Pass 2 is complete
    //     unsafe {
    //         gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
    //     }
    //     unsafe {
    //         gl::MemoryBarrier(gl::ALL_BARRIER_BITS);
    //     }
        
    //     // Pass 3: Accumulate face normals to vertex normals
    //     unsafe {
    //         comp_shader.set_uniform1i("pass", &3);
    //     }
        
    //     comp_shader.dispatch_compute(triangle_work_groups as u32, 1, 1);

    //     //let mut face_normals = vec![[0.0f32; 3]; triangle_count];
    //     // face_normals_buffer.bind();
    //     // unsafe {
    //     //     gl::GetBufferSubData(
    //     //         gl::SHADER_STORAGE_BUFFER,
    //     //         0,
    //     //         (triangle_count * 3 * std::mem::size_of::<f32>()) as isize,
    //     //         face_normals.as_mut_ptr() as *mut std::ffi::c_void,
    //     //     );
    //     // }

    //     // // CPU-side accumulation (Pass 3 substitute)
    //     // let mut vertex_normal_accum = vec![[0.0f32; 3]; vertex_count];
    //     // let mut vertex_triangle_counts = vec![0i32; vertex_count];

    //     // for i in 0..triangle_count {
    //     //     let i0 = indices[i * 3] as usize;
    //     //     let i1 = indices[i * 3 + 1] as usize;
    //     //     let i2 = indices[i * 3 + 2] as usize;

    //     //     let normal = face_normals[i];

    //     //     for &idx in &[i0, i1, i2] {
    //     //         vertex_normal_accum[idx][0] += normal[0];
    //     //         vertex_normal_accum[idx][1] += normal[1];
    //     //         vertex_normal_accum[idx][2] += normal[2];
    //     //         vertex_triangle_counts[idx] += 1;
    //     //     }
    //     // }

    //     // // Upload CPU-accumulated data to GPU
    //     // vertex_normals_buffer.bind();
    //     // let flat_normals: Vec<f32> = vertex_normal_accum.iter().flat_map(|n| n.iter().copied()).collect();
    //     // vertex_normals_buffer.store_f32_data(&flat_normals);

    //     // vertex_counts_buffer.bind();
    //     // vertex_counts_buffer.store_i32_data(&vertex_triangle_counts);
        
    //     // Ensure Pass 3 is complete
    //     unsafe {
    //         gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
    //     }
    //     unsafe {
    //         gl::MemoryBarrier(gl::ALL_BARRIER_BITS);
    //     }
        
    //     // Pass 4: Normalize vertex normals
    //     unsafe {
    //         comp_shader.set_uniform1i("pass", &4);
    //     }
        
    //     comp_shader.dispatch_compute(vertex_work_groups as u32, 1, 1);
        
    //     // Ensure Pass 4 is complete
    //     unsafe {
    //         gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
    //     }
    //     unsafe {
    //         gl::MemoryBarrier(gl::ALL_BARRIER_BITS);
    //     }
    //     // Read the calculated normals
    //     vertex_normals_buffer.bind();
    //     let mut calculated_normals = vec![0.0f32; vertex_count * 3];
        
    //     unsafe {
    //         gl::GetBufferSubData(
    //             gl::SHADER_STORAGE_BUFFER,
    //             0,
    //             (vertex_count * 3 * std::mem::size_of::<f32>()) as isize,
    //             calculated_normals.as_mut_ptr() as *mut std::ffi::c_void
    //         );
    //     }

    //     assert_eq!(vertices.len(), vertex_count * stride, "Vertex array size mismatch");
    //     assert_eq!(calculated_normals.len(), vertex_count * 3, "Normal array mismatch");
        
    //     // Copy the normals back into the original vertex data idk if I should do this in cpu or GPU tbh i dont even know if I should be doing this... TODO check this out
    //     if vertices.len() >= vertex_count * stride {
    //         for i in 0..vertex_count {
    //             let base = i * stride + normal_offset;
    //             vertices[base]     = calculated_normals[i * 3];
    //             vertices[base + 1] = calculated_normals[i * 3 + 1];
    //             vertices[base + 2] = calculated_normals[i * 3 + 2];
    //         }
    //     } else {
    //         panic!("Vertex buffer is smaller than expected for writing normals");
    //     }
        

    //     for i in 0..10 {
    //         println!(
    //             "v{}: pos=({}, {}, {}) norm=({}, {}, {})",
    //             i,
    //             vertices[i * stride],
    //             vertices[i * stride + 1],
    //             vertices[i * stride + 2],
    //             vertices[i * stride + normal_offset],
    //             vertices[i * stride + normal_offset + 1],
    //             vertices[i * stride + normal_offset + 2],
    //         );
    //     }

    //     println!("First few calculated normals:");
    //     for i in 0..10 {
    //         println!("Vertex {}: Normal ({}, {}, {})", 
    //             i,
    //             calculated_normals[i * 3],
    //             calculated_normals[i * 3 + 1],
    //             calculated_normals[i * 3 + 2]
    //         );
    //     }

    //     let max_index = *indices.iter().max().unwrap();
    //     println!("Max index referenced: {}", max_index);
    //     println!("Vertex count: {}", vertex_count);

        
    //     // Update the VBO with the new vertex data (including normals)
    //     self.vbo.bind();
    //     self.vbo.store_f32_data(vertices);
        
    //     // Since we're using the same VBO and the attribute pointers are already set up,
    //     // we don't need to modify the VAO
    // }



