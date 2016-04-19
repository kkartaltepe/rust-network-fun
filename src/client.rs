extern crate gl;
extern crate glutin;
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
    let mut graphix = Renderer::init("Client Window");
    let mut simulation = Simulation::init();

    print!("Done.\n");
    // Do Simulation and rendering
    println!("Beginning simulation");
    let mut should_close = false;

    while !should_close {
        let _ = socket.recv_from(&mut buf).unwrap(); // Get the Hello back
        simulation.deserialize(&buf);
        println!("Finished deserializing data from the server");
        simulation.step();
        println!("Finished simulation step");

        unsafe { // Opengl calls are unsafe
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::ClearColor(0.38, 0.906, 0.722, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        for geom_data in simulation.geoms.iter() {
            graphix.render_cube(geom_data.0); // Seperate from ODE?
        }

        for event in graphix.window.poll_events() {
            handle_window_event(event, &mut should_close);
        }

        graphix.window.swap_buffers().unwrap();
    }

    //Do clean ups
    graphix.clean_up();
    simulation.clean_up();

}

fn handle_window_event(event: glutin::Event, should_close: &mut bool) {
    use glutin::Event;
    use glutin::ElementState as KeyState;
    use glutin::VirtualKeyCode as Key;

    match event {
        Event::KeyboardInput(KeyState::Pressed, _, Some(Key::Q)) => {
            *should_close = true;
        }
        _ => ()
    }
}
