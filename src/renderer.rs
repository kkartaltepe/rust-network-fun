extern crate glfw;
extern crate gl;
extern crate libc;
extern crate ode;

use shader_loader;

use std::mem;
use std::ffi::CString;
use std::sync::mpsc::Receiver;
use std::f32::*;
use std;
use glfw::{Context, WindowHint};
use gl::types::*;
use vec::Vec3;
use ode::*;

const IDENT_MAT : [f32; 16] = [
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0
];

static CUBE_VERTEX_DATA : [f32; 108] = [
    -0.5,-0.5,-0.5, // triangle 1 : begin
    -0.5,-0.5, 0.5,
    -0.5, 0.5, 0.5, // triangle 1 : end
    0.5, 0.5,-0.5, // triangle 2 : begin
    -0.5,-0.5,-0.5,
    -0.5, 0.5,-0.5, // triangle 2 : end
    0.5,-0.5, 0.5,
    -0.5,-0.5,-0.5,
    0.5,-0.5,-0.5,
    0.5, 0.5,-0.5,
    0.5,-0.5,-0.5,
    -0.5,-0.5,-0.5,
    -0.5,-0.5,-0.5,
    -0.5, 0.5, 0.5,
    -0.5, 0.5,-0.5,
    0.5,-0.5, 0.5,
    -0.5,-0.5, 0.5,
    -0.5,-0.5,-0.5,
    -0.5, 0.5, 0.5,
    -0.5,-0.5, 0.5,
    0.5,-0.5, 0.5,
    0.5, 0.5, 0.5,
    0.5,-0.5,-0.5,
    0.5, 0.5,-0.5,
    0.5,-0.5,-0.5,
    0.5, 0.5, 0.5,
    0.5,-0.5, 0.5,
    0.5, 0.5, 0.5,
    0.5, 0.5,-0.5,
    -0.5, 0.5,-0.5,
    0.5, 0.5, 0.5,
    -0.5, 0.5,-0.5,
    -0.5, 0.5, 0.5,
    0.5, 0.5, 0.5,
    -0.5, 0.5, 0.5,
    0.5,-0.5, 0.5
];

static CUBE_NORMAL_DATA : [f32; 108] = [
    -1.0, 0.0, 0.0, // triangle 1 : begin
    -1.0, 0.0, 0.0,
    -1.0, 0.0, 0.0, // triangle 1 : end
    0.0, 0.0,-1.0, // triangle 2 : begin
    0.0, 0.0,-1.0,
    0.0, 0.0,-1.0, // triangle 2 : end
    0.0,-1.0, 0.0,
    0.0,-1.0, 0.0,
    0.0,-1.0, 0.0,
    0.0, 0.0,-1.0,
    0.0, 0.0,-1.0,
    0.0, 0.0,-1.0,
    -1.0, 0.0, 0.0,
    -1.0, 0.0, 0.0,
    -1.0, 0.0, 0.0,
    0.0,-1.0, 0.0,
    0.0,-1.0, 0.0,
    0.0,-1.0, 0.0,
    0.0, 0.0, 1.0,
    0.0, 0.0, 1.0,
    0.0, 0.0, 1.0,
    1.0, 0.0, 0.0,
    1.0, 0.0, 0.0,
    1.0, 0.0, 0.0,
    1.0, 0.0, 0.0,
    1.0, 0.0, 0.0,
    1.0, 0.0, 0.0,
    0.0, 1.0, 0.0,
    0.0, 1.0, 0.0,
    0.0, 1.0, 0.0,
    0.0, 1.0, 0.0,
    0.0, 1.0, 0.0,
    0.0, 1.0, 0.0,
    0.0, 0.0, 1.0,
    0.0, 0.0, 1.0,
    0.0, 0.0, 1.0
];

fn perspective(matrix : &mut [f32; 16], near : f32, far : f32, width : f32, height : f32) {
    let ar = width/height;
    let alpha = 95.0*consts::PI/180.0; // 95* FoV
    let halftan = (alpha/2.0).tan();
    *matrix = [
        1.0/(ar*halftan), 0.0, 0.0, 0.0,
        0.0, 1.0/halftan, 0.0, 0.0,
        0.0, 0.0, (near-far)/(far-near), (2.0*near*far)/(near-far),
        0.0, 0.0, -1.0, 0.0
    ];
}

fn look_at(matrix : &mut [f32; 16], target : Vec3, eye : Vec3, up : Vec3) {
    let l = (eye - target).normalize();
    let s = up.cross(l).normalize();
    let nup = l.cross(s).normalize();
    *matrix = [
        s.x, nup.x, l.x,  0.0,
        s.y, nup.y, l.y,  0.0,
        s.z, nup.z, l.z,  0.0,
        -eye.dot(s), -eye.dot(nup), -eye.dot(l), 1.0
    ];
}

fn translate(matrix: &mut [f32; 16], x: f32, y: f32, z: f32) {
    (*matrix)[3] += z;
    (*matrix)[7] += y;
    (*matrix)[11] += x;
}




pub struct Renderer {
    pub window: glfw::Window,
    pub events: Receiver<(f64, glfw::WindowEvent)>,
    shaders: Vec<GLuint>,
    program_id: GLuint,
    model_mat: [f32; 16],
    view_mat: [f32; 16],
    proj_mat: [f32; 16],
    vert_array_id: GLuint,
    vertexbuffer: GLuint,
    normalbuffer: GLuint,
}


impl Renderer {
    pub fn init() -> Renderer{
        let mut ret;
        unsafe {
            let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
            glfw.window_hint(WindowHint::ContextVersion(3, 2));
            let (mut window, events) = glfw.create_window(600, 600, "Server Window", glfw::WindowMode::Windowed)
                .expect("Failed to create GLFW window.");

            window.set_key_polling(true);
            window.make_current();

            //Loads all GL functions
            gl::load_with(|s| window.get_proc_address(s));

            ret = Renderer{
                window: window,
                events: events,
                shaders: Vec::new(),
                program_id: 0,
                model_mat: IDENT_MAT,
                view_mat: IDENT_MAT,
                proj_mat: IDENT_MAT,
                vert_array_id: 0,
                vertexbuffer: 0,
                normalbuffer: 0,
            };
            ret.shaders.push(shader_loader::compile_shader_file("vertex.glsl", gl::VERTEX_SHADER));
            ret.shaders.push(shader_loader::compile_shader_file("frag.glsl", gl::FRAGMENT_SHADER));
            ret.program_id = shader_loader::link_program(ret.shaders[0], ret.shaders[1]);
            gl::UseProgram(ret.program_id);

            perspective(&mut ret.proj_mat, 0.1, 400.0, 1.0, 1.0);
            look_at(&mut ret.view_mat,
                    Vec3::new(0.0, 0.0, -1.0),
                    Vec3::new(0.0, 3.0, -7.0),
                    Vec3::new(0.0, 1.0, 0.0)
                    );

            let view_id = gl::GetUniformLocation(ret.program_id, CString::new("view").unwrap().as_ptr());
            let proj_id = gl::GetUniformLocation(ret.program_id, CString::new("proj").unwrap().as_ptr());

            gl::UniformMatrix4fv(view_id, 1, gl::FALSE, &ret.view_mat[0]);
            gl::UniformMatrix4fv(proj_id, 1, gl::TRUE, &ret.proj_mat[0]);

            gl::GenVertexArrays(1, &mut ret.vert_array_id);
            gl::BindVertexArray(ret.vert_array_id);

            gl::GenBuffers(1, &mut ret.vertexbuffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, ret.vertexbuffer);
            gl::BufferData(gl::ARRAY_BUFFER,
                           mem::size_of_val(&CUBE_VERTEX_DATA) as i64,
                           CUBE_VERTEX_DATA.as_ptr() as *const libc::c_void,
                           gl::STATIC_DRAW);

            gl::GenBuffers(1, &mut ret.normalbuffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, ret.normalbuffer);
            gl::BufferData(gl::ARRAY_BUFFER,
                           mem::size_of_val(&CUBE_NORMAL_DATA) as i64,
                           CUBE_NORMAL_DATA.as_ptr() as *const libc::c_void,
                           gl::STATIC_DRAW);
        }

        return ret;
    }

    pub fn clean_up(&self) {
        unsafe {
            gl::DeleteProgram(self.program_id);
            gl::DeleteShader(self.shaders[0]);
            gl::DeleteShader(self.shaders[1]);
            gl::DeleteBuffers(1, &self.vertexbuffer);
            gl::DeleteBuffers(1, &self.normalbuffer);
            gl::DeleteVertexArrays(1, &self.vert_array_id);
        }
    }


    pub fn render_cube(&mut self, geom: dGeomID) {
        unsafe {
            let rot_mat: *const f32 = ode::dGeomGetRotation(geom);
            let pos_vec: *const f32 = ode::dGeomGetPosition(geom);
            let rust_pos = std::slice::from_raw_parts(pos_vec, 3);
            let rust_rot = std::slice::from_raw_parts(rot_mat, 16);
            for i in 0..12 {
                self.model_mat[i] = rust_rot[i];
            }
            translate(&mut self.model_mat, rust_pos[0], rust_pos[1], rust_pos[2]);
            let model_mat_id = gl::GetUniformLocation(self.program_id, CString::new("model").unwrap().as_ptr());
            gl::UniformMatrix4fv(model_mat_id, 1, gl::TRUE, &self.model_mat[0]);

            let position_loc = gl::GetAttribLocation(self.program_id,
                                                     CString::new("ms_position").unwrap().as_ptr()) as GLuint;
            gl::EnableVertexAttribArray(position_loc); // Corresponds to location = X in vert_shader
            // Can be programatic using GetAttribLocation
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertexbuffer);
            gl::VertexAttribPointer(
                position_loc,
                3,
                gl::FLOAT,
                gl::FALSE,
                0,
                std::ptr::null()
                );
            let normal_loc = gl::GetAttribLocation(self.program_id,
                                                   CString::new("ms_normal").unwrap().as_ptr()) as GLuint;
            gl::EnableVertexAttribArray(normal_loc); // Corresponds to location = X in vert_shader
            gl::BindBuffer(gl::ARRAY_BUFFER, self.normalbuffer);
            gl::VertexAttribPointer(
                normal_loc,
                3,
                gl::FLOAT,
                gl::FALSE,
                0,
                std::ptr::null()
                );

            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            gl::DisableVertexAttribArray(position_loc);
            gl::DisableVertexAttribArray(normal_loc);
        }
    }

    #[allow(dead_code)]
    fn print_matrix(name: String, matrix: [f32; 16]) {
        println!("{}[] = [", name);
        for i in 0..16 {
            print!("{}, ", matrix[i]);
            if (i+1)%4 == 0 {
                print!("\n");
            }
        }
        println!("]");
    }
}
