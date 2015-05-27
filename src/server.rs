#![feature(convert)]
extern crate glfw;
extern crate gl;
extern crate libc;

mod shader_loader;
mod vec;

use std::net::UdpSocket;
use std::thread;
use std::mem;
use std::ffi::CString;
use std::f32::*;
use glfw::{Action, Context, Key, WindowHint};
use gl::types::*;
use vec::Vec3;

static VERTEX_DATA : [f32; 9] = [
    -1.0, -1.0, -1.0,
    1.0, -1.0, -1.0,
    0.0, 1.0, -1.0
];

static CUBE_VERTEX_DATA : [f32; 108] = [
    -1.0,-1.0,-1.0, // triangle 1 : begin
    -1.0,-1.0, 1.0,
    -1.0, 1.0, 1.0, // triangle 1 : end
    1.0, 1.0,-1.0, // triangle 2 : begin
    -1.0,-1.0,-1.0,
    -1.0, 1.0,-1.0, // triangle 2 : end
    1.0,-1.0, 1.0,
    -1.0,-1.0,-1.0,
    1.0,-1.0,-1.0,
    1.0, 1.0,-1.0,
    1.0,-1.0,-1.0,
    -1.0,-1.0,-1.0,
    -1.0,-1.0,-1.0,
    -1.0, 1.0, 1.0,
    -1.0, 1.0,-1.0,
    1.0,-1.0, 1.0,
    -1.0,-1.0, 1.0,
    -1.0,-1.0,-1.0,
    -1.0, 1.0, 1.0,
    -1.0,-1.0, 1.0,
    1.0,-1.0, 1.0,
    1.0, 1.0, 1.0,
    1.0,-1.0,-1.0,
    1.0, 1.0,-1.0,
    1.0,-1.0,-1.0,
    1.0, 1.0, 1.0,
    1.0,-1.0, 1.0,
    1.0, 1.0, 1.0,
    1.0, 1.0,-1.0,
    -1.0, 1.0,-1.0,
    1.0, 1.0, 1.0,
    -1.0, 1.0,-1.0,
    -1.0, 1.0, 1.0,
    1.0, 1.0, 1.0,
    -1.0, 1.0, 1.0,
    1.0,-1.0, 1.0
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
    let mut normalbuffer = 0;
    let mut program_id;
    let mut vert_shader;
    let mut frag_shader;
    unsafe {
        gl::GenVertexArrays(1, &mut vert_array_id);
        gl::BindVertexArray(vert_array_id);

        gl::GenBuffers(1, &mut vertexbuffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, vertexbuffer);
        gl::BufferData(gl::ARRAY_BUFFER,
                       mem::size_of_val(&CUBE_VERTEX_DATA) as i64,
                       CUBE_VERTEX_DATA.as_ptr() as *const libc::c_void,
                       gl::STATIC_DRAW);

        gl::GenBuffers(1, &mut normalbuffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, normalbuffer);
        gl::BufferData(gl::ARRAY_BUFFER,
                       mem::size_of_val(&CUBE_NORMAL_DATA) as i64,
                       CUBE_NORMAL_DATA.as_ptr() as *const libc::c_void,
                       gl::STATIC_DRAW);


        vert_shader = shader_loader::compile_shader_file("vertex.glsl", gl::VERTEX_SHADER);
        frag_shader = shader_loader::compile_shader_file("frag.glsl", gl::FRAGMENT_SHADER);
        program_id = shader_loader::link_program(vert_shader, frag_shader);
        gl::UseProgram(program_id);

        perspective(&mut PROJ_MAT, 0.1, 1000.0, 1.0, 1.0);
        look_at(&mut VIEW_MAT,
                Vec3::new(0.0, 0.0, -1.0),
                Vec3::new(4.0, 3.0, -3.0),
                Vec3::new(0.0, 1.0, 0.0)
                );

        let view_id = gl::GetUniformLocation(program_id, CString::new("view").unwrap().as_ptr());
        let proj_id = gl::GetUniformLocation(program_id, CString::new("proj").unwrap().as_ptr());

        gl::UniformMatrix4fv(view_id, 1, gl::FALSE, &VIEW_MAT[0]);
        gl::UniformMatrix4fv(proj_id, 1, gl::TRUE, &PROJ_MAT[0]);
    }
    print!("Done.\n");

    while !window.should_close() {
        glfw.poll_events();
        unsafe { // Opengl calls are unsafe
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::ClearColor(0.38, 0.906, 0.722, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            let position_loc = gl::GetAttribLocation(program_id,
                                                     CString::new("ms_position").unwrap().as_ptr()) as GLuint;
            gl::EnableVertexAttribArray(position_loc); // Corresponds to location = X in vert_shader
                                            // Can be programatic using GetAttribLocation
            gl::BindBuffer(gl::ARRAY_BUFFER, vertexbuffer);
            gl::VertexAttribPointer(
                position_loc,
                3,
                gl::FLOAT,
                gl::FALSE,
                0,
                std::ptr::null()
            );
            let normal_loc = gl::GetAttribLocation(program_id,
                                                     CString::new("ms_normal").unwrap().as_ptr()) as GLuint;
            gl::EnableVertexAttribArray(normal_loc); // Corresponds to location = X in vert_shader
            gl::BindBuffer(gl::ARRAY_BUFFER, normalbuffer);
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
