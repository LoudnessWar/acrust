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
    pub fn new_normals(vertices: &[f32], indices: &[i32]) -> Self {
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
        println!("mesh normals: {:?}", mesh.normals.as_ref().unwrap());

        println!("mesh normals:");
        mesh 
    }

    pub fn get_vao(&self) -> &Vao{
        &self.vao
    }

    pub fn calculate_normals(&mut self, vertices: &[f32], indices: &[i32]) {
        let vertex_count = vertices.len() / 6; // Assuming 6 floats per vertex (pos + something else)
        let triangle_count = indices.len() / 3;
        
        // Create buffers for the multi-pass approach
        
        // 1. Face normals buffer (temporary)
        let face_normals_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_DRAW);
        face_normals_buffer.bind();
        let face_normal_data = vec![0.0f32; triangle_count * 3]; // 3 components per face normal
        face_normals_buffer.store_f32_data(&face_normal_data);
        
        // 2. Vertex normals buffer (output)
        let vertex_normals_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_DRAW);
        vertex_normals_buffer.bind();
        let vertex_normal_data = vec![0.0f32; vertex_count * 3]; // 3 components per vertex normal
        vertex_normals_buffer.store_f32_data(&vertex_normal_data);
        
        // 3. Vertex triangle counts buffer
        print!("herhe");
        let vertex_counts_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_DRAW);
        vertex_counts_buffer.bind();
        let vertex_count_data = vec![0i32; vertex_count]; // One int per vertex
        vertex_counts_buffer.store_i32_data(&vertex_count_data);
        
        // Set up and run compute shader
        let mut comp_shader = ShaderProgram::new_compute("shaders/normals.comp");
        comp_shader.bind();

        comp_shader.create_uniforms(vec!["vertex_count", "index_count", "pass"]);
        
        // Common uniform setup
        comp_shader.set_uniform1i("vertex_count", &(vertex_count as i32));
        comp_shader.set_uniform1i("index_count", &(indices.len() as i32));
        
        // Calculate work group sizes
        let work_group_size = 256; // Typical compute shader workgroup size
        let triangle_work_groups = (triangle_count + work_group_size - 1) / work_group_size;
        let vertex_work_groups = (vertex_count + work_group_size - 1) / work_group_size;
        
        // Pass 1: Calculate face normals
        print!("Calculate face normals");
        unsafe {
            // Bind buffers to the compute shader
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, self.vbo.get_id());
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, self.ebo.get_id());
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, face_normals_buffer.get_id());
            check_gl_error("poop 2");
            comp_shader.set_uniform1i("pass", &1);
        }
        
        comp_shader.dispatch_compute(triangle_work_groups as u32, 1, 1);
        
        // Ensure Pass 1 is complete
        unsafe {
            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
        }
        
        // Pass 2: Initialize vertex normals and counting buffers
        unsafe {
            println!("Initialize vertex normals and counting buffers");
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 3, vertex_normals_buffer.get_id());
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 4, vertex_counts_buffer.get_id());
            check_gl_error("poop");
            comp_shader.set_uniform1i("pass", &2);
        }
        
        comp_shader.dispatch_compute(vertex_work_groups as u32, 1, 1);
        
        // Ensure Pass 2 is complete
        unsafe {
            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
        }
        
        // Pass 3: Accumulate face normals to vertex normals
        unsafe {
            comp_shader.set_uniform1i("pass", &3);
        }
        
        comp_shader.dispatch_compute(triangle_work_groups as u32, 1, 1);
        
        // Ensure Pass 3 is complete
        unsafe {
            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
        }
        check_gl_error("unmapped poop 5");
        // Pass 4: Normalize vertex normals
        unsafe {
            comp_shader.set_uniform1i("pass", &4);
        }
        
        comp_shader.dispatch_compute(vertex_work_groups as u32, 1, 1);
        
        // Ensure Pass 4 is complete
        unsafe {
            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
        }
        
        check_gl_error("unmapped poop 56");
        println!("Map the normals buffer to read the results");
        // Map the normals buffer to read the results
        vertex_normals_buffer.bind();
        unsafe {
            let size = (vertex_count * 3 * std::mem::size_of::<f32>()) as isize;
            let ptr = gl::MapBufferRange(
                gl::SHADER_STORAGE_BUFFER, 
                0,
                size,
                gl::MAP_READ_BIT
            ) as *const f32;
            check_gl_error("unmapped poop 4");
            if !ptr.is_null() {
                // Copy data from GPU to CPU
                let normals_slice = std::slice::from_raw_parts(ptr, vertex_count * 3);
                self.normals = Some(normals_slice.to_vec());
                
                // Unmap the buffer
                gl::UnmapBuffer(gl::SHADER_STORAGE_BUFFER);
                check_gl_error("unmapped poop");
            }

            
        }
        check_gl_error("unmapped poop 2");
        // Store the normals buffer in the struct
        self.normals_buffer = Some(vertex_normals_buffer);
        check_gl_error("unmapped poop 3");
        
        // Clean up temporary buffers
        // Note: We don't need to explicitly delete face_normals_buffer and vertex_counts_buffer
        // as they will be automatically cleaned up when they go out of scope
        
        // Now update your VAO to include normals
        self.vao.bind();
        
        //check_gl_error("vnormals_buffer");
        if let Some(buffer) = &self.normals_buffer {
            buffer.bind();
            
            check_gl_error("poop inside");
            // Set up the normal attribute (assuming it's attribute 1)
            VertexAttribute::new(
                1, // attribute index for normals
                3, // 3 components for normals
                gl::FLOAT,
                gl::FALSE,
                3 * std::mem::size_of::<f32>() as i32, // stride (3 floats per normal)
                std::ptr::null()
            ).enable();
        }
        
        self.vao.unbind();
    }
    

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
}

fn check_gl_error(label: &str) {
    let err = unsafe { gl::GetError() };
    if err != gl::NO_ERROR {
        panic!("GL Error after {}: 0x{:X}", label, err);
    }
}


//i want to add a 3d object trait here with a basic render and like basic funciton ect ect

//ok so i did what i said above but in objload which uuuh might change its location yall

