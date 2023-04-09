pub mod shader {
    use crate::gl;
    use std::fs::File;
    use std::io::prelude::*;
    use std::ptr;
    use std::rc::Rc;
    use std::str;

    const LOG_SIZE: usize = 1024;

    struct Shader {
        gl: Rc<gl::Gl>,
        shader: gl::types::GLuint,
    }

    impl Shader {
        pub fn new(gl: Rc<gl::Gl>, shader_type: gl::types::GLenum) -> Self {
            assert!([gl::VERTEX_SHADER, gl::FRAGMENT_SHADER].contains(&shader_type));

            let shader = unsafe { gl.CreateShader(shader_type) };
            Shader { gl, shader }
        }

        pub fn compile(self, source: &[u8]) -> Self {
            let gl = &self.gl;
            let shader = self.shader;

            let mut success: i32 = 0;
            let mut info_log = [0; LOG_SIZE];

            unsafe {
                gl.ShaderSource(shader, 1, [source.as_ptr().cast()].as_ptr(), ptr::null());
                gl.CompileShader(shader);

                gl.GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
            }

            if success == 0 {
                unsafe {
                    gl.GetShaderInfoLog(shader, LOG_SIZE as i32, ptr::null_mut(), info_log.as_mut_ptr());
                }
                println!(
                    "{}",
                    str::from_utf8(&info_log.iter().take_while(|&i| *i > 0).map(|i| *i as u8).collect::<Vec<u8>>()).unwrap()
                );
            }

            self
        }
    }

    impl Drop for Shader {
        fn drop(&mut self) {
            unsafe {
                self.gl.DeleteShader(self.shader);
            }
        }
    }

    pub struct Program {
        gl: Rc<gl::Gl>,
        program: gl::types::GLuint,
    }

    impl Program {
        pub fn new(gl: Rc<gl::Gl>) -> Self {
            let program = unsafe { gl.CreateProgram() };
            Program { gl, program }
        }

        pub fn link(self, vertex_file: &str, fragment_file: &str) -> Self {
            let gl = &self.gl;
            let program = self.program;

            let mut vertex_source = Vec::new();
            File::open(vertex_file).unwrap().read_to_end(&mut vertex_source).unwrap();
            vertex_source.push(0);

            let mut fragment_source = Vec::new();
            File::open(fragment_file).unwrap().read_to_end(&mut fragment_source).unwrap();
            fragment_source.push(0);

            let vertex_shader = Shader::new(Rc::clone(&gl), gl::VERTEX_SHADER).compile(&vertex_source);
            let fragment_shader = Shader::new(Rc::clone(&gl), gl::FRAGMENT_SHADER).compile(&fragment_source);

            let mut success: i32 = 0;

            unsafe {
                gl.AttachShader(program, vertex_shader.shader);
                gl.AttachShader(program, fragment_shader.shader);
                gl.LinkProgram(program);
                gl.DetachShader(program, vertex_shader.shader);
                gl.DetachShader(program, fragment_shader.shader);

                gl.GetProgramiv(program, gl::LINK_STATUS, &mut success);
            }

            if success == 0 {
                let mut info_log = [0; LOG_SIZE];
                unsafe {
                    gl.GetProgramInfoLog(program, LOG_SIZE as i32, ptr::null_mut(), info_log.as_mut_ptr());
                }
                println!(
                    "{}",
                    str::from_utf8(&info_log.iter().take_while(|&i| *i > 0).map(|i| *i as u8).collect::<Vec<u8>>()).unwrap()
                );
            }

            self
        }

        pub fn apply(&self) {
            unsafe {
                self.gl.UseProgram(self.program);
            }
        }

        pub fn set_int(&self, name: &str, value: i32) {
            let gl = &self.gl;
            let id = self.program;

            let c_name = name
                .as_bytes()
                .iter()
                .filter(|&u| *u < 128u8)
                .map(|u| *u as i8)
                .chain(std::iter::once(0))
                .collect::<Vec<i8>>();

            unsafe {
                gl.Uniform1i(gl.GetUniformLocation(id, c_name.as_ptr()), value);
            }
        }
    }

    impl Drop for Program {
        fn drop(&mut self) {
            unsafe {
                self.gl.DeleteProgram(self.program);
            }
        }
    }
}
