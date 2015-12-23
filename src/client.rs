extern crate glfw;
extern crate gl;
extern crate libc;
extern crate ode;
extern crate byteorder;

mod shader_loader;
mod renderer;
mod simulation;
mod vec;

use std::net::UdpSocket;
use renderer::Renderer;
use simulation::Simulation;
use glfw::{Action, Context, Key};

fn main() {
    print!("Starting client . . . ");
    let socket = UdpSocket::bind("127.0.0.1:35556").unwrap();

    let mut buf = [0; 9000];
    let _ = socket.send_to(&buf, "127.0.0.1:35555").unwrap();
    let (amt, _) = socket.recv_from(&mut buf).unwrap(); // Get the Hello back
    println!("Connecting to server.");
    println!("Recieved {} bytes hello from server.", amt);
    //println!("Sent {}/{}[{}%] bytes", sent, buf.len(), (sent/buf.len()) as u32);

    //Init everything
    let mut graphix = Renderer::init();
    let mut simulation = Simulation::init();

    print!("Done.\n");
    // Do Simulation and rendering
    println!("Beginning simulation");
    while !graphix.window.should_close() {
        graphix.window.glfw.poll_events();
        unsafe { // Opengl calls are unsafe

            let _ = socket.recv_from(&mut buf).unwrap(); // Get the Hello back
            simulation.deserialize(&buf);
            simulation.step();

            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::ClearColor(0.38, 0.906, 0.722, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            for geom_data in simulation.geoms.iter() {
                graphix.render_cube(geom_data.0); // Seperate from ODE?
            }
        }

        for(_, event) in glfw::flush_messages(&graphix.events) {
            handle_window_event(event, &mut graphix.window);
        }

        graphix.window.swap_buffers();
    }

    //Do clean ups
    graphix.clean_up();
    simulation.clean_up();

}

fn handle_window_event(event: glfw::WindowEvent, window: &mut glfw::Window) {
    match event {
        glfw::WindowEvent::Key(Key::Q, _, Action::Press, _) => {
            window.set_should_close(true);
        }
        _ => ()
    }
}
