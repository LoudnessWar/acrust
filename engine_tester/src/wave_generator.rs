use acrust::graphics::gl_wrapper::*;
use gl::types::*;
use std::mem;
use std::ptr;

use crate::octo::OctreeNode;

pub struct WaterRender {
    vao: Vao,
    vbo: BufferObject,
    ebo: BufferObject,
    index_count: i32,
}

impl WaterRender {
    pub fn new(length: f32, width: f32, quadSize: f32) -> Self {
        let mut vertices: Vec<f32> = Vec::new();//uuuh this is lazy lol
        let mut indices: Vec<i32> = Vec::new();
        let mut offset = 0;
        let mut uvs = Vec::new();//literally not even used

        for i in 0..length as i32
        {
            for j in 0..width as i32
            {
                vertices.extend(vec![j as f32 * quadSize, 0.0, i as f32 * quadSize]);
                uvs.extend(vec![j as f32 / width, i as f32 / width]);
            }
        }

        for  i in 0..length as i32//jaaaa this is like haskell or something
        {
            for j in 0..width as i32
            {
                let start = i * (width as i32 + 1) + j;
                let nextRow = start + width as i32 + 1;

                indices.push(start);//hmmm idk if this will work better the what is above ie
                indices.push(nextRow);//adding them to a like vector and then just appending that, the one benefit might be how
                indices.push(start + 1);//the memory is saved up there might be less confusing

                indices.push(start + 1);
                indices.push(nextRow);
                indices.push(nextRow + 1);//ok so push doesnt that like not work here ... idk it probably adds it to the wong place
            }
        }

        println!("{:?}", vertices);
        println!("{:?}", indices);


        let vao = Vao::new();
        vao.bind();

        let vbo = BufferObject::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW);
        vbo.bind();
        vbo.store_f32_data(&vertices);

        let ebo = BufferObject::new(gl::ELEMENT_ARRAY_BUFFER, gl::STATIC_DRAW);
        ebo.bind();
        ebo.store_i32_data(&indices);

        let stride = 6 * mem::size_of::<GLfloat>() as GLsizei;//this uuuh is fine...nah but i am curious
        VertexAttribute::new(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null()).enable();//verticies/indies
        //VertexAttribute::new(1, 3, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<GLfloat>()) as *const GLvoid).enable();//color

        WaterRender {
            vao,
            vbo,
            ebo,
            index_count: indices.len() as i32,
        }
    }

    pub fn render(&self) {
        self.vao.bind();
        unsafe {
            gl::DrawElements(gl::TRIANGLES, self.index_count, gl::UNSIGNED_INT, ptr::null());
        }
    }
}
