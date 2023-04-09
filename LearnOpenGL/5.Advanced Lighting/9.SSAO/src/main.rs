use gfx_maths::{mat4::Mat4, quaternion::Quaternion, vec2::Vec2, vec3::Vec3};
use glfw::{Action, Context, CursorMode, Key, Window, WindowEvent, WindowHint};
use std::{f32::consts::PI, fs::File, io::Read, mem::size_of, ptr, rc::Rc};

const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;
const SCR_NEAR: f32 = 0.1;
const SCR_FAR: f32 = 50.0;
const SSAO_KERNEL: usize = 64usize;

pub mod camera;
use camera::camera::Camera;
use camera::camera::Movement;
pub mod mesh;
use mesh::mesh::{Mesh, Texture, Vertex, VertexArray};
pub mod model;
use model::model::Model;
pub mod shader;
use shader::shader::Program;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(WindowHint::ContextVersionMajor(3));
    glfw.window_hint(WindowHint::ContextVersionMinor(3));
    glfw.window_hint(WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    let (mut window, events) = glfw
        .create_window(SCR_WIDTH, SCR_HEIGHT, "LearnOpenGL", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.make_current();
    window.set_cursor_mode(CursorMode::Disabled);
    window.set_framebuffer_size_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_scroll_polling(true);
    window.focus();

    let gl = Rc::new(gl::Gl::load_with(|s| window.get_proc_address(s).cast()));

    gl.depth_enable(true);

    let shader_geometry = Program::new(Rc::clone(&gl)).link("src/9.ssao_geometry.vs", "src/9.ssao_geometry.fs");
    let shader_lighting = Program::new(Rc::clone(&gl)).link("src/9.ssao.vs", "src/9.ssao_lighting.fs");
    let shader_ssao = Program::new(Rc::clone(&gl)).link("src/9.ssao.vs", "src/9.ssao.fs");
    let shader_ssao_blur = Program::new(Rc::clone(&gl)).link("src/9.ssao.vs", "src/9.ssao_blur.fs");

    let backpack = Model::new(Rc::clone(&gl)).load_model("resources/objects/backpack/backpack.obj");

    let cube = VertexArray::new_cube(Rc::clone(&gl));
    let quad = VertexArray::new_quad(Rc::clone(&gl));

    let g_buffer = Framebuffer::new_g_buffer(Rc::clone(&gl));
    let ssao = Framebuffer::new_single(Rc::clone(&gl));
    let ssao_blur = Framebuffer::new_single(Rc::clone(&gl));

    let mut ssao_kernel = Vec::new();
    for i in 0..SSAO_KERNEL {
        let mut sample = Vec3::new(fastrand::f32() * 2.0 - 1.0, fastrand::f32() * 2.0 - 1.0, fastrand::f32());
        sample.normalize();
        sample *= fastrand::f32();
        let mut scale = i as f32 / SSAO_KERNEL as f32;
        scale = our_lerp(0.1, 1.0, scale * scale);
        sample *= scale;
        ssao_kernel.push(sample);
    }

    let mut ssao_noise = Vec::new();
    for _ in 0..16 {
        ssao_noise.push(fastrand::f32() * 2.0 - 1.0);
        ssao_noise.push(fastrand::f32() * 2.0 - 1.0);
        ssao_noise.push(0.0);
    }
    let noise_tex = Texture::new(Rc::clone(&gl), "", "").load_noise(ssao_noise);

    let light_pos = Vec3::new(2.0, 4.0, -2.0);
    let light_color = Vec3::new(0.2, 0.2, 0.7);

    shader_lighting.apply();
    shader_lighting.set_int("gPosition", 0);
    shader_lighting.set_int("gNormal", 1);
    shader_lighting.set_int("gAlbedo", 2);
    shader_lighting.set_int("ssao", 3);
    shader_ssao.apply();
    shader_ssao.set_int("gPosition", 0);
    shader_ssao.set_int("gNormal", 1);
    shader_ssao.set_int("texNoise", 2);
    shader_ssao_blur.apply();
    shader_ssao_blur.set_int("ssaoInput", 0);

    glfw.poll_events();

    let (x_pos, y_pos) = window.get_cursor_pos();
    let mut camera = Camera::new(Vec3::new(0.0, 0.0, 5.0), x_pos as f32, y_pos as f32);

    let mut last_frame = 0.0;

    while !window.should_close() {
        let current_frame = glfw.get_time() as f32;
        let delta_time = current_frame - last_frame;
        last_frame = current_frame;

        process_input(&mut camera, &mut window, delta_time);

        gl.clear_color(0.0, 0.0, 0.0, 1.0);

        g_buffer.bind();
        gl.clear();
        let projection = Mat4::perspective_opengl(camera.zoom(), SCR_NEAR, SCR_FAR, SCR_WIDTH as f32 / SCR_HEIGHT as f32);
        let view = camera.view_matrix();
        shader_geometry.apply();
        shader_geometry.set_mat4("projection", projection);
        shader_geometry.set_mat4("view", view);
        let model = Mat4::translate(Vec3::new(0.0, 7.0, 0.0)) * Mat4::scale(Vec3::new(7.5, 7.5, 7.5));
        shader_geometry.set_mat4("model", model);
        shader_geometry.set_int("invertedNormals", 1);
        cube.bind();
        cube.draw();
        shader_geometry.set_int("invertedNormals", 0);
        let model = Mat4::translate(Vec3::new(0.0, 0.5, 0.0)) * Mat4::rotate(Quaternion::axis_angle(Vec3::new(1.0, 0.0, 0.0), -PI / 2.0));
        shader_geometry.set_mat4("model", model);
        backpack.draw(&shader_geometry);
        gl.unbind_framebuffer();

        ssao.bind();
        gl.clear();
        shader_ssao.apply();
        for i in 0..SSAO_KERNEL {
            shader_ssao.set_vec3(&("samples[".to_string() + &(i.to_string() + "]")), *ssao_kernel.get(i).unwrap());
        }
        shader_ssao.set_mat4("projection", projection);
        gl.active_texture(0);
        g_buffer.bind_texture(0);
        gl.active_texture(1);
        g_buffer.bind_texture(1);
        gl.active_texture(2);
        noise_tex.bind();
        quad.bind();
        quad.draw();
        gl.unbind_framebuffer();

        ssao_blur.bind();
        gl.clear();
        shader_ssao_blur.apply();
        gl.active_texture(0);
        ssao.bind_texture(0);
        quad.draw();
        gl.unbind_framebuffer();

        gl.clear();
        shader_lighting.apply();
        let lpv = view * light_pos.extend(1.0);
        shader_lighting.set_vec3("light.Position", Vec3::new(lpv.x, lpv.y, lpv.z));
        shader_lighting.set_vec3("light.Color", light_color);
        let linear = 0.09;
        let quadratic = 0.032;
        shader_lighting.set_float("light.Linear", linear);
        shader_lighting.set_float("light.Quadratic", quadratic);
        gl.active_texture(0);
        g_buffer.bind_texture(0);
        gl.active_texture(1);
        g_buffer.bind_texture(1);
        gl.active_texture(2);
        g_buffer.bind_texture(2);
        gl.active_texture(3);
        ssao_blur.bind_texture(0);
        quad.draw();

        window.swap_buffers();

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&gl, &mut camera, &mut window, event);
        }
    }
}

fn our_lerp(a: f32, b: f32, f: f32) -> f32 {
    a + f * (b - a)
}

fn process_input(camera: &mut Camera, window: &mut Window, delta_time: f32) {
    if window.get_key(Key::Escape) == Action::Press {
        window.set_should_close(true);
    }

    if window.get_key(Key::W) == Action::Press {
        camera.process_keyboard(Movement::Forward, delta_time);
    }
    if window.get_key(Key::S) == Action::Press {
        camera.process_keyboard(Movement::Backward, delta_time);
    }
    if window.get_key(Key::A) == Action::Press {
        camera.process_keyboard(Movement::Left, delta_time);
    }
    if window.get_key(Key::D) == Action::Press {
        camera.process_keyboard(Movement::Right, delta_time);
    }
}

fn handle_window_event(gl: &gl::Gl, camera: &mut Camera, window: &mut glfw::Window, event: glfw::WindowEvent) {
    match event {
        WindowEvent::FramebufferSize(width, height) => unsafe {
            gl.Viewport(0, 0, width, height);
        },
        WindowEvent::Scroll(_, y_offset) => camera.process_scroll(y_offset as f32),
        WindowEvent::CursorPos(_x_pos, _y_pos) => {
            let (x_pos, y_pos) = window.get_cursor_pos();

            camera.process_mouse(x_pos as f32, y_pos as f32, true);
        }
        _ => {}
    }
}

mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    impl self::Gl {
        pub fn clear_color(&self, red: f32, green: f32, blue: f32, alpha: f32) {
            unsafe {
                self.ClearColor(red, green, blue, alpha);
            }
        }

        pub fn clear(&self) {
            unsafe {
                self.Clear(self::COLOR_BUFFER_BIT | self::DEPTH_BUFFER_BIT);
            }
        }

        pub fn unbind_vao(&self) {
            unsafe {
                self.BindVertexArray(0);
            }
        }

        pub fn unbind_framebuffer(&self) {
            unsafe {
                self.BindFramebuffer(self::FRAMEBUFFER, 0);
            }
        }

        pub fn depth_enable(&self, on: bool) {
            unsafe {
                if on {
                    self.Enable(self::DEPTH_TEST);
                } else {
                    self.Disable(self::DEPTH_TEST);
                }
            }
        }

        pub fn active_texture(&self, unit: u32) {
            assert!(unit <= self::MAX_COMBINED_TEXTURE_IMAGE_UNITS);

            unsafe {
                self.ActiveTexture(self::TEXTURE0 + unit);
            }
        }
    }

    pub fn c_name(name: &str) -> Vec<i8> {
        c_finish(&mut name.as_bytes().iter())
    }

    pub fn c_named(name: &str, number: &str) -> Vec<i8> {
        c_finish(&mut name.as_bytes().iter().chain(number.as_bytes().iter()))
    }

    fn c_finish(iter: &mut dyn Iterator<Item = &u8>) -> Vec<i8> {
        iter.filter(|&u| *u < 128u8).map(|u| *u as i8).chain(std::iter::once(0)).collect::<Vec<_>>()
    }
}

const FB_MAX_TEXAS: usize = 3usize;

pub struct Framebuffer {
    gl: Rc<gl::Gl>,
    framebuffer: gl::types::GLuint,
    textures: [gl::types::GLuint; FB_MAX_TEXAS],
    rbo: gl::types::GLuint,
}

impl Framebuffer {
    pub fn new_g_buffer(gl: Rc<gl::Gl>) -> Self {
        let mut framebuffer = 0;
        let mut textures = [0, 0, 0];
        let mut rbo = 0;

        unsafe {
            gl.GenFramebuffers(1, &mut framebuffer);
            gl.GenTextures(FB_MAX_TEXAS as i32, textures.as_mut_ptr());
            gl.GenRenderbuffers(1, &mut rbo);

            gl.BindFramebuffer(gl::FRAMEBUFFER, framebuffer);

            gl.BindTexture(gl::TEXTURE_2D, textures[0]);
            gl.TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA16F as i32,
                SCR_WIDTH as i32,
                SCR_HEIGHT as i32,
                0,
                gl::RGBA,
                gl::FLOAT,
                ptr::null(),
            );
            let nearest = gl::NEAREST as i32;
            let clamp_edge = gl::CLAMP_TO_EDGE as i32;
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, nearest);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, nearest);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, clamp_edge);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, clamp_edge);

            gl.FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0 as u32, gl::TEXTURE_2D, textures[0], 0);

            gl.BindTexture(gl::TEXTURE_2D, textures[1]);
            gl.TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA16F as i32,
                SCR_WIDTH as i32,
                SCR_HEIGHT as i32,
                0,
                gl::RGBA,
                gl::FLOAT,
                ptr::null(),
            );
            let nearest = gl::NEAREST as i32;
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, nearest);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, nearest);

            gl.FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1 as u32, gl::TEXTURE_2D, textures[1], 0);

            gl.BindTexture(gl::TEXTURE_2D, textures[2]);
            gl.TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                SCR_WIDTH as i32,
                SCR_HEIGHT as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                ptr::null(),
            );
            let nearest = gl::NEAREST as i32;
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, nearest);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, nearest);

            gl.FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT2 as u32, gl::TEXTURE_2D, textures[2], 0);

            let attachments = [gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1, gl::COLOR_ATTACHMENT2];
            gl.DrawBuffers(FB_MAX_TEXAS as i32, attachments.as_ptr());

            gl.BindRenderbuffer(gl::RENDERBUFFER, rbo);
            gl.RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT, SCR_WIDTH as i32, SCR_HEIGHT as i32);
            gl.FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, rbo);

            if gl.CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                println!("ERROR::FRAMEBUFFER:: Framebuffer is not complete!")
            }
            gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        Framebuffer { gl, framebuffer, textures, rbo }
    }

    pub fn new_single(gl: Rc<gl::Gl>) -> Self {
        let mut framebuffer = 0;
        let mut textures = [0, 0, 0];

        unsafe {
            gl.GenFramebuffers(1, &mut framebuffer);
            gl.GenTextures(1, textures.as_mut_ptr());

            gl.BindFramebuffer(gl::FRAMEBUFFER, framebuffer);

            gl.BindTexture(gl::TEXTURE_2D, textures[0]);
            gl.TexImage2D(gl::TEXTURE_2D, 0, gl::RED as i32, SCR_WIDTH as i32, SCR_HEIGHT as i32, 0, gl::RED, gl::FLOAT, ptr::null());
            let nearest = gl::NEAREST as i32;
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, nearest);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, nearest);

            gl.FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, textures[0], 0);

            if gl.CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                println!("ERROR::FRAMEBUFFER:: Framebuffer is not complete!")
            }
            gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        Framebuffer {
            gl,
            framebuffer,
            textures,
            rbo: 0,
        }
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
        }
    }

    pub fn bind_texture(&self, texture: usize) {
        assert!(texture < FB_MAX_TEXAS);

        unsafe {
            self.gl.BindTexture(gl::TEXTURE_2D, self.textures[texture]);
        }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        let gl = &self.gl;

        unsafe {
            gl.DeleteFramebuffers(1, &self.framebuffer);
            gl.DeleteTextures(FB_MAX_TEXAS, self.textures.as_ptr());
            gl.DeleteRenderbuffers(1, &self.rbo);
        }
    }
}
