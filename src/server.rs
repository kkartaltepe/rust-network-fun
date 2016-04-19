//extern crate glfw;
extern crate gl;
extern crate glutin;
extern crate libc;
extern crate ode;
extern crate byteorder;
extern crate time;

mod shader_loader;
mod renderer;
mod simulation;
mod vec;

use std::net::UdpSocket;
use vec::Vec3;
use renderer::Renderer;
use simulation::Simulation;
use time::{Duration, PreciseTime};

//static VERTEX_DATA : [f32; 9] = [
    //-1.0, -1.0, -1.0,
    //1.0, -1.0, -1.0,
    //0.0, 1.0, -1.0
//];

fn format_bytes(bytes: u64) -> String {
    let suffixes = [ "B", "KB", "MB", "GB" ];
    let multiplier = [ 1024u64, 1024, 1024 ];
    let mut suffix = 0;
    let mut sigdig = bytes;

    while sigdig > 10 * multiplier[suffix] &&
          suffix < multiplier.len() {
        sigdig /= multiplier[suffix];
        suffix += 1;
    }
    return format!("{}{}", sigdig, suffixes[suffix]);
}

fn main() {
    print!("Starting server . . . ");
    let socket = UdpSocket::bind("127.0.0.1:35555").unwrap();
    let mut buf = [0; 9000];
    println!("Waiting on client.");
    let (_, client) = socket.recv_from(&mut buf).unwrap(); //Receive into the buffer
    println!("Client connected from {}.", client);

    //Init everything
    let mut graphix = Renderer::init("Server Window");
    let mut simulation = Simulation::init();

    simulation.create_cube(10.0, Vec3::new(0.0, 1.0, 0.0));
    for n in 0..100 {
        simulation.create_cube(1.0, Vec3::new(((n/10)*2) as f32, 3.0, ((n%10)*2) as f32));
    }
    print!("Done.\n");

    // Do Simulation and rendering
    println!("Beginning simulation");
    let mut bytes_sent = 0u64;
    let mut last_second = PreciseTime::now();
    let mut should_close = false;
    while !should_close {
        bytes_sent += socket.send_to(&simulation.serialize(), client).unwrap() as u64;
        let now = PreciseTime::now();
        let differential = last_second.to(now);
        if differential > Duration::seconds(1) {
            println!("Sent {} in {} second.", format_bytes(bytes_sent), differential);
            bytes_sent = 0;
            last_second = now;
        }

        unsafe { // Opengl calls are unsafe
            simulation.step();

            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::ClearColor(0.38, 0.906, 0.722, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            for geom_data in simulation.geoms.iter() {
                graphix.render_cube(geom_data.0); // Seperate from ODE?
            }
        }

        for event in graphix.window.poll_events() {
            handle_window_event(event, &mut simulation, &mut should_close)
        }

        graphix.window.swap_buffers().unwrap();
    }

    //Do clean ups
    graphix.clean_up();
    simulation.clean_up();
}

fn handle_window_event(event: glutin::Event, simulation: &mut Simulation, should_close: &mut bool ) {
    use glutin::Event;
    use glutin::ElementState as KeyState;
    use glutin::VirtualKeyCode as Key;

    let push_force = 500f32;
    match event {
        Event::KeyboardInput(KeyState::Pressed, _, Some(Key::Q)) => {
            *should_close = true;
        }
        Event::KeyboardInput(KeyState::Pressed, _, Some(Key::P)) => {
            simulation.toggle_pause();
        }
        Event::KeyboardInput(KeyState::Pressed, _, Some(Key::Up)) => {
            simulation.apply_force(simulation.geoms[0].0, Vec3::new(0.0, 0.0, push_force));
        }
        Event::KeyboardInput(KeyState::Pressed, _, Some(Key::Down)) => {
            simulation.apply_force(simulation.geoms[0].0, Vec3::new(0.0, 0.0, -push_force));
        }
        Event::KeyboardInput(KeyState::Pressed, _, Some(Key::Left)) => {
            simulation.apply_force(simulation.geoms[0].0, Vec3::new(push_force, 0.0, 0.0));
        }
        Event::KeyboardInput(KeyState::Pressed, _, Some(Key::Right)) => {
            simulation.apply_force(simulation.geoms[0].0, Vec3::new(-push_force, 0.0, 0.0));
        }
        _ => ()
    }
}
