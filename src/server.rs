#![feature(convert)]
extern crate glfw;
extern crate gl;
extern crate libc;

mod shader_loader;

use std::net::UdpSocket;
use std::thread;
use std::mem;
use glfw::{Action, Context, Key, WindowHint, OpenGlProfileHint};
use gl::types::*;
//use shared_loader;

// Shader sources
static VS_SRC: &'static str =
   "#version 150\n\
    in vec2 position;\n\
    void main() {\n\
       gl_Position = vec4(position, 0.0, 1.0);\n\
    }";

static FS_SRC: &'static str =
   "#version 150\n\
    out vec4 out_color;\n\
    void main() {\n\
       out_color = vec4(1.0, 1.0, 1.0, 1.0);\n\
    }";

fn main() {
    print!("Starting server . . . ");
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(WindowHint::ContextVersion(3, 2));
    //glfw.window_hint(WindowHint::OpenglProfile(OpenGlProfileHint::Core));
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
    let triangle_data : [f32; 9] = [
        -1.0, -1.0, 0.0,
        1.0, -1.0, 0.0,
        0.0, 1.0, 0.0
    ];
    let mut vert_array_id = 0;
    let mut vertexbuffer = 0;
    let mut program_id = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vert_array_id);
        gl::BindVertexArray(vert_array_id);
        gl::GenBuffers(1, &mut vertexbuffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, vertexbuffer);
        gl::BufferData(gl::ARRAY_BUFFER, mem::size_of_val(&triangle_data) as i64, triangle_data.as_ptr() as *const libc::c_void, gl::STATIC_DRAW);
        let vert_shader = shader_loader::compile_shader(VS_SRC, gl::VERTEX_SHADER);
        let frag_shader = shader_loader::compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
        program_id = shader_loader::link_program(vert_shader, frag_shader);
    }
    print!("Done.\n");

    while !window.should_close() {
        glfw.poll_events();
        unsafe { // Opengl calls are unsafe
            gl::ClearColor(0.38, 0.906, 0.722, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(program_id);

            gl::EnableVertexAttribArray(0);
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
            gl::DisableVertexAttribArray(0);
        }

        for(_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event);
        }

        window.swap_buffers();
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
