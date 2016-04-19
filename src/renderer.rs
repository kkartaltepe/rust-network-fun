#![allow(dead_code)]

extern crate gl;
extern crate glutin;
extern crate libc;
extern crate ode;

use shader_loader;

use std::mem;
use std::ffi::CString;
use std::f32::*;
use std;
// use glfw::{Context, WindowHint};
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
    //*matrix = [
        //s.x, nup.x, l.x,  0.0,
        //s.y, nup.y, l.y,  0.0,
        //s.z, nup.z, l.z,  0.0,
        //-eye.dot(s), -eye.dot(nup), -eye.dot(l), 1.0
    //];
    *matrix = [
        s.x, s.y, s.z,  -eye.dot(s),
        nup.x, nup.y, nup.z,  -eye.dot(nup),
        l.x, l.y, l.z,  -eye.dot(l),
        0.0, 0.0, 0.0, 1.0
    ];
}

//Valid for ROW MAJOR matrices
fn rtranslate(m: &mut [f32; 16], x: f32, y: f32, z: f32) {
    (*m)[3] += (*m)[0]*x+(*m)[1]*y+(*m)[2]*z;
    (*m)[7] += (*m)[4]*x+(*m)[5]*y+(*m)[6]*z;
    (*m)[11] += (*m)[8]*x+(*m)[9]*y+(*m)[10]*z;
}

fn ltranslate(m: &mut [f32; 16], x: f32, y: f32, z: f32) {
    (*m)[3] += x;
    (*m)[7] += y;
    (*m)[11] += z;
}





pub struct Renderer {
    pub window: glutin::Window,
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
    pub fn init(win_name: &str) -> Renderer{
        let mut ret;
        unsafe {
            //let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
            //glfw.window_hint(WindowHint::ContextVersion(3, 2));
            //let (mut window, events) = glfw.create_window(600, 600, win_name, glfw::WindowMode::Windowed)
                //.expect("Failed to create GLFW window.");

            // window.set_key_polling(true);
            // window.make_current();

            let window = glutin::WindowBuilder::new()
                                    .with_title(win_name.to_string())
                                    .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3,3)))
                                    .with_vsync()
                                    .build()
                                    .unwrap();

            // It is essential to make the context current before calling `gl::load_with`.
            window.make_current().unwrap();

            //Loads all GL functions
            gl::load_with(|s| window.get_proc_address(s) as *const _);

            ret = Renderer{
                window: window,
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
                    Vec3::new(10.0, 0.0, 10.0),
                    Vec3::new(10.0, 2.0, -10.0),
                    Vec3::new(0.0, 1.0, 0.0)
                    );

            let view_id = gl::GetUniformLocation(ret.program_id, CString::new("view").unwrap().as_ptr());
            let proj_id = gl::GetUniformLocation(ret.program_id, CString::new("proj").unwrap().as_ptr());

            //Take the transpose of ALL matrices before feeding them to opengl because I am working
            //with ROW MAJOR matrices.
            gl::UniformMatrix4fv(view_id, 1, gl::TRUE, &ret.view_mat[0]);
            gl::UniformMatrix4fv(proj_id, 1, gl::TRUE, &ret.proj_mat[0]);

            gl::GenVertexArrays(1, &mut ret.vert_array_id);
            gl::BindVertexArray(ret.vert_array_id);

            gl::GenBuffers(1, &mut ret.vertexbuffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, ret.vertexbuffer);
            gl::BufferData(gl::ARRAY_BUFFER,
                           mem::size_of_val(&CUBE_VERTEX_DATA) as isize,
                           CUBE_VERTEX_DATA.as_ptr() as *const _,
                           gl::STATIC_DRAW);

            gl::GenBuffers(1, &mut ret.normalbuffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, ret.normalbuffer);
            gl::BufferData(gl::ARRAY_BUFFER,
                           mem::size_of_val(&CUBE_NORMAL_DATA) as isize,
                           CUBE_NORMAL_DATA.as_ptr() as *const _,
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
            let pos = std::slice::from_raw_parts(ode::dGeomGetPosition(geom), 3);
            let rot = std::slice::from_raw_parts(ode::dGeomGetRotation(geom), 12); // 3 rows, 4 columns.
            self.model_mat = IDENT_MAT;
            for i in 0..3 {
                self.model_mat[i*4] = rot[i*4];
                self.model_mat[i*4+1] = rot[i*4+1];
                self.model_mat[i*4+2] = rot[i*4+2];
            }
            ltranslate(&mut self.model_mat, pos[0], pos[1], pos[2]);
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
