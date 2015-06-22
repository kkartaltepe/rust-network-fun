#![feature(convert)]
extern crate glfw;
extern crate gl;
extern crate libc;
extern crate ode;
extern crate byteorder;
extern crate time;

mod shader_loader;
mod renderer;
mod simulation;
mod vec;

use std::net::UdpSocket;
use glfw::{Action, Context, Key};
use vec::Vec3;
use renderer::Renderer;
use simulation::Simulation;
use time::{Duration, PreciseTime};

//static VERTEX_DATA : [f32; 9] = [
    //-1.0, -1.0, -1.0,
    //1.0, -1.0, -1.0,
    //0.0, 1.0, -1.0
//];

fn main() {
    print!("Starting server . . . ");
    let socket = UdpSocket::bind("127.0.0.1:35555").unwrap();
    let mut buf = [0; 100];
    println!("Waiting on client.");
    let (_, client) = socket.recv_from(&mut buf).unwrap(); //Receive into the buffer
    println!("Client connected from {}.", client);

    //Init everything
    let mut graphix = Renderer::init();
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
    while !graphix.window.should_close() {
        bytes_sent += socket.send_to(&simulation.serialize(), client).unwrap() as u64;
        let now = PreciseTime::now();
        let differential = last_second.to(now);
        if differential > Duration::seconds(1) {
            println!("Sent {} bytes in {} second.", bytes_sent, differential);
            bytes_sent = 0;
            last_second = now;
        }

        graphix.window.glfw.poll_events();
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

        for(_, event) in glfw::flush_messages(&graphix.events) {
            handle_window_event(event, &mut graphix.window, &mut simulation);
        }

        graphix.window.swap_buffers();
    }

    //Do clean ups
    graphix.clean_up();
    simulation.clean_up();
}

fn handle_window_event(event: glfw::WindowEvent, window: &mut glfw::Window, simulation: &mut Simulation ) {
    let push_force = 500f32;
    match event {
        glfw::WindowEvent::Key(Key::Q, _, Action::Press, _) => {
            window.set_should_close(true);
        }
        glfw::WindowEvent::Key(Key::P, _, Action::Press, _) => {
            simulation.toggle_pause();
        }
        glfw::WindowEvent::Key(Key::Up, _, _, _) => {
            simulation.apply_force(simulation.geoms[0].0, Vec3::new(0.0, 0.0, push_force));
        }
        glfw::WindowEvent::Key(Key::Down, _, _, _) => {
            simulation.apply_force(simulation.geoms[0].0, Vec3::new(0.0, 0.0, -push_force));
        }
        glfw::WindowEvent::Key(Key::Left, _, _, _) => {
            simulation.apply_force(simulation.geoms[0].0, Vec3::new(push_force, 0.0, 0.0));
        }
        glfw::WindowEvent::Key(Key::Right, _, _, _) => {
            simulation.apply_force(simulation.geoms[0].0, Vec3::new(-push_force, 0.0, 0.0));
        }
        _ => ()
    }
}
