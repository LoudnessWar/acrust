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

//ok so the mesh right should have the normals?
//yeaaahhhh
pub struct Mesh {
    vao: Vao,
    vbo: BufferObject,
    ebo: BufferObject,
    index_count: i32,
    normals: Option<BufferObject>,//every mesh needs like vertices and like indicies and stuff yada yada but not every mesh will like need normals
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
            normals: None,//im storing these for now for uuuhhh like debugging purpouses
        }
    }

    pub fn new_normals(vertices: &[f32], indices: &[i32]) -> Self {
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
        VertexAttribute::new(1, 3, gl::FLOAT, gl::FALSE, 6 * mem::size_of::<GLfloat>() as i32, (3 * mem::size_of::<GLfloat>()) as *const GLvoid).enable();

        vao.unbind();//just to be safe added unbind

        Self {
            vao,
            vbo,
            ebo,
            index_count: indices.len() as i32,
            normals: None,
        }
    }

    pub fn get_vao(&self) -> &Vao{
        &self.vao
    }

    pub fn calculate_normals(&mut self, vertices: &[f32], indices: &[i32]){
        //le psudo code for getting normals
        // let edge1 = B - A;
        // let edge2 = C - A;
        // let face_normal = edge1.cross(edge2).normalize();

        //so we need like the verticies and indicies... I think my best like option for this would to be to store it in the BufferObjectThemselves... The problem is that the values can get really big so I think it will probably not work out the best
        //in the long run so this should be passed them instead. I cam probably instead then create seperate consturctors for if you want normals or not

        //ok compute shader is just better for this





        //below is cool and all but would rather use compute shader
        // let (mut edge1, mut edge2): (f32, f32);
        // let mut face_normal;


        // // //dude like... is this even efficient. I just like this stuff
        // // let map= indices.chunks_exact(3).enumerate().map(|(i, xyz)| (i, vertices.chunks_exact(3).map(|verts| ([verts[0] as usize], vertices[verts[1] as usize], vertices[verts[2] as usize]))));
        // let vertex_map: Result<Vec<_>, _> = indices.chunks_exact(3)
        //     .enumerate()
        //     .map(|(group_idx, idx_group)| {
        //         let get_vertex = |idx| {
        //             let pos = (idx as usize) * 3;
        //             if pos + 2 >= vertices.len() {
        //                 Err(format!("Vertex index {} out of bounds", idx))
        //             } else {
        //                 Ok((vertices[pos], vertices[pos+1], vertices[pos+2]))
        //             }
        //         };
                
        //         Ok((
        //             group_idx,
        //             get_vertex(idx_group[0])?,
        //             get_vertex(idx_group[1])?,
        //             get_vertex(idx_group[2])?
        //         ))
        //     }).collect();

        

        //)//[xyz[0] as usize], vertices[xyz[1] as usize], vertices[xyz[2] as usize])).map(|(i, x, y ,z)| 
        // {   edge1 = y - x;
        //     edge2 = z - x;
        //     face_normal = edge1.cross(edge2).normalize()
        // });

        // for (i, x, y, z) in map{
        //     edge1 = y - x;
        //     edge2 = z - x;
        //     face_normal = edge1.cross(edge2).normalize()
        // }
        // for (x, y ,z) in indices{
        //     xp = vertices.get(x);
        //     yp = vertices.get(y);
        //     zp = vertices.get(z);

            
        // }

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
}

//i want to add a 3d object trait here with a basic render and like basic funciton ect ect

//ok so i did what i said above but in objload which uuuh might change its location yall

