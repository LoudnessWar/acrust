
pub struct ShaderProgram {
    program_handle: u32,
    uniform_ids: HashMap<String, GLint>,
}

#[allow(temporary_cstring_as_ptr)]
impl ShaderProgram {
    pub fn new(vertex_shader_path: &str, fragment_shader_path: &str) -> Self {
        let vertex_shader_source = Self::read_shader_source(vertex_shader_path);
        let fragment_shader_source = Self::read_shader_source(fragment_shader_path);
        let program_handle = unsafe {
            let vertex_shader = Self::compile_shader(&vertex_shader_source, gl::VERTEX_SHADER);
            let fragment_shader = Self::compile_shader(&fragment_shader_source, gl::FRAGMENT_SHADER);
            let handle = gl::CreateProgram();
            gl::AttachShader(handle, vertex_shader);
            gl::AttachShader(handle, fragment_shader);
            gl::LinkProgram(handle);
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
            handle
        };

        ShaderProgram {
            program_handle,
            uniform_ids: HashMap::new(),
        }
    }

    fn read_shader_source(path: &str) -> String {
        let mut file = File::open(path).unwrap_or_else(|_| panic!("Failed to open {}", path));
        let mut source = String::new();
        file.read_to_string(&mut source)
            .expect("Failed to read shader file");
        source
    }

    fn compile_shader(source: &str, shader_type: GLenum) -> GLuint {
        let shader = unsafe { gl::CreateShader(shader_type) };
        let c_str = CString::new(source).unwrap();
        unsafe {
            gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
            gl::CompileShader(shader);
        }
        shader
    }

    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.program_handle);
        }
    }

    pub fn unbind() {
        unsafe {
            gl::UseProgram(0);
        }
    }

    pub fn create_uniform(&mut self, uniform_name: &str) {//all this really does is like init a uniform and check if your shader actually like need it
        let uniform_location = unsafe {
            gl::GetUniformLocation(
                self.program_handle,
                CString::new(uniform_name).unwrap().as_ptr(),
            )
        };
        if uniform_location < 0 {
            panic!("Cannot locate uniform: {} \n    or issue with frament shader", uniform_name);
        } else {
            self.uniform_ids.insert(uniform_name.to_string(), uniform_location);
        }
    }

    pub fn set_matrix4fv_uniform(&self, uniform_name: &str, matrix: &Matrix4<f32>) {
        unsafe {
            gl::UniformMatrix4fv(
                self.uniform_ids[uniform_name],
                1,
                gl::FALSE,
                matrix.as_ptr(),
            )
        }
    }

    pub fn set_uniform1i(&self, uniform_name: &str, value: &i32) {
        unsafe {
            gl::Uniform1iv(
                self.uniform_ids[uniform_name],
                1,
                value,
            )
        }
    }

    pub fn set_uniform4f(&self, uniform_name: &str, value: &Vector4<f32>) {
        unsafe {
            gl::Uniform4fv(
                self.uniform_ids[uniform_name],
                1,
                value.as_ptr(),
            )
        }
    }

    pub fn enable_depth(&self) {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
        }
    }

    pub fn enable_backface_culling(&self) {
        unsafe {
            gl::Enable(gl::CULL_FACE);     // Enable face culling
            gl::CullFace(gl::BACK);        // Cull back faces
            //gl::FrontFace(gl::CCW);        // Use counter-clockwise vertex winding for front faces
        }
    }

    pub fn get_program_handle(&self) -> u32 {
        self.program_handle
    }
}