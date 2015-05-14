#![feature(convert)]
extern crate glfw;
extern crate gl;
extern crate libc;

mod shader_loader;

use std::net::UdpSocket;
use std::thread;
use std::mem;
use std::ffi::CString;
use std::ops::{Add, Sub, Neg};
use std::f32::*;
use glfw::{Action, Context, Key, WindowHint, OpenGlProfileHint};
use gl::types::*;

static VERTEX_DATA : [f32; 9] = [
    -1.0, -1.0, -1.0,
    1.0, -1.0, -1.0,
    0.0, 1.0, -1.0
];

static mut VIEW_MAT : [f32; 16] = [
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0
];

static mut PROJ_MAT : [f32; 16] = [
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0
];

#[derive(Copy, Clone)]
struct Vec3 {
    pub x : f32,
    pub y : f32,
    pub z : f32
}

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, other: Vec3) -> Vec3 {
     return Vec3 {
         x: self.x + other.x,
         y: self.y + other.y,
         z: self.z + other.z
     }
    }
}

impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Vec3 {
     return Vec3 {
         x: -self.x,
         y: -self.y,
         z: -self.z
     }
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, other: Vec3) -> Vec3 {
     return Vec3 {
         x: self.x - other.x,
         y: self.y - other.y,
         z: self.z - other.z
     }
    }
}

impl Vec3 {
    fn new(x_: f32, y_: f32, z_: f32) -> Vec3 {
        return Vec3 {
            x: x_,
            y: y_,
            z: z_
        }
    }

    fn normalize(mut self) -> Vec3 {
        let len = self.dot(self).sqrt();
        self.x = self.x/len;
        self.y = self.y/len;
        self.z = self.z/len;
        self
    }

    fn cross(self, b : Vec3) -> Vec3 {
        return Vec3 {
            x: self.y*b.z - self.z*b.y,
            y: self.z*b.x - self.x*b.z,
            z: self.x*b.y - self.y*b.x
        }
    }

    fn dot(self, other : Vec3) -> f32 {
     return (self.x * other.x)
            + (self.y * other.y)
            + (self.z * other.z)
    }
}

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
    let L = (eye - target).normalize();
    let S = up.cross(L).normalize();
    let up_ = L.cross(S).normalize();
    *matrix = [
        S.x, up_.x, L.x,  0.0,
        S.y, up_.y, L.y,  0.0,
        S.z, up_.z, L.z,  0.0,
        -eye.dot(S), -eye.dot(up_), -eye.dot(L), 1.0
    ];
}


fn main() {
    print!("Starting server . . . ");
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(WindowHint::ContextVersion(3, 2));
    let (mut window, events) = glfw.create_window(300, 300, "Server Window", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");
    let socket = UdpSocket::bind("127.0.0.1:34555").unwrap();

    thread::spawn(move || {
        let mut buf = [0; 100];
        println!("Waiting on data.");
        let (amt, src) = socket.recv_from(&mut buf).unwrap(); //Receive into the buffer
        println!("Recieved {} bytes from {}.", amt, src);

        drop(socket);
    });

    window.set_key_polling(true);
    window.make_current();
    //Loads all GL functions
    gl::load_with(|s| window.get_proc_address(s));

    let mut vert_array_id = 0;
    let mut vertexbuffer = 0;
    let mut program_id;
    let mut vert_shader;
    let mut frag_shader;
    unsafe {
        gl::GenVertexArrays(1, &mut vert_array_id);
        gl::BindVertexArray(vert_array_id);

        gl::GenBuffers(1, &mut vertexbuffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, vertexbuffer);
        gl::BufferData(gl::ARRAY_BUFFER,
                       mem::size_of_val(&VERTEX_DATA) as i64,
                       VERTEX_DATA.as_ptr() as *const libc::c_void,
                       gl::STATIC_DRAW);

        vert_shader = shader_loader::compile_shader_file("vertex.glsl", gl::VERTEX_SHADER);
        frag_shader = shader_loader::compile_shader_file("frag.glsl", gl::FRAGMENT_SHADER);
        program_id = shader_loader::link_program(vert_shader, frag_shader);
        gl::UseProgram(program_id);

        perspective(&mut PROJ_MAT, 0.1, 1000.0, 1.0, 1.0);
        look_at(&mut VIEW_MAT,
                Vec3::new(0.0, 0.0, -1.0),
                Vec3::new(4.0, 3.0, 3.0),
                Vec3::new(0.0, 1.0, 0.0)
                );

        //for fl in PROJ_MAT.iter() {
            //println!("{} ", fl);
        //}

        let view_id = gl::GetUniformLocation(program_id, CString::new("view").unwrap().as_ptr());
        let proj_id = gl::GetUniformLocation(program_id, CString::new("proj").unwrap().as_ptr());

        gl::UniformMatrix4fv(view_id, 1, gl::FALSE, &VIEW_MAT[0]);
        gl::UniformMatrix4fv(proj_id, 1, gl::TRUE, &PROJ_MAT[0]);
    }
    print!("Done.\n");

    while !window.should_close() {
        glfw.poll_events();
        unsafe { // Opengl calls are unsafe
            gl::ClearColor(0.38, 0.906, 0.722, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            let position_loc = gl::GetAttribLocation(program_id,
                                                     CString::new("ms_position").unwrap().as_ptr()) as GLuint;
            gl::EnableVertexAttribArray(position_loc); // Corresponds to location = X in vert_shader
                                            // Can be programatic using GetAttribLocation
            gl::BindBuffer(gl::ARRAY_BUFFER, vertexbuffer);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                0,
                std::ptr::null()
            );

            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::DisableVertexAttribArray(position_loc);
        }

        for(_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event);
        }

        window.swap_buffers();
    }

    unsafe {
        gl::DeleteProgram(program_id);
        gl::DeleteShader(vert_shader);
        gl::DeleteShader(frag_shader);
        gl::DeleteBuffers(1, &vertexbuffer);
        gl::DeleteVertexArrays(1, &vert_array_id);
    }
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::Key(Key::Q, _, Action::Press, _) => {
            window.set_should_close(true)
        }
        _ => ()
    }
}
