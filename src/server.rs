#![feature(convert)]
extern crate glfw;
extern crate gl;
extern crate libc;
extern crate ode;

mod shader_loader;
mod renderer;
mod vec;

use std::net::UdpSocket;
use std::thread;
use glfw::{Action, Context, Key};
use vec::Vec3;
use ode::*;
use renderer::Renderer;

//static VERTEX_DATA : [f32; 9] = [
    //-1.0, -1.0, -1.0,
    //1.0, -1.0, -1.0,
    //0.0, 1.0, -1.0
//];

struct WorldData {
    world : ode::dWorldID,
    contacts : ode::dJointGroupID,
}

extern fn near_callback(data : *mut libc::c_void, obj1 : ode::dGeomID, obj2 : ode::dGeomID) {
    unsafe {
        let world_data: &mut WorldData = std::mem::transmute(data);
        let b1 = dGeomGetBody(obj1);
        let b2 = dGeomGetBody(obj2);
        let mut contact = ode::dContact{..Default::default()};
        contact.surface.mode = (ode::dContactBounce | ode::dContactSoftCFM) as i32;
        contact.surface.mu = ode::dInfinity;
        contact.surface.bounce = 0.6;
        contact.surface.bounce_vel = 0.5;
        contact.surface.soft_cfm = 0.001;
        if ode::dCollide(obj1, obj2, 1, &mut contact.geom, std::mem::size_of::<ode::dContact>() as libc::c_int) != 0 {
            //println!("Collision detected!");
            let joint = ode::dJointCreateContact(world_data.world, world_data.contacts, &mut contact);
            ode::dJointAttach(joint, b1, b2);
        }
    }
}

fn create_cube(location: Vec3, world: dWorldID, space: dSpaceID) -> (dGeomID, Box<dMass>) {
    let body;
    let geom;
    let mut m: Box<ode::dMass>;
    unsafe {
        body = ode::dBodyCreate(world);
        geom = dCreateBox(space, 1.0, 1.0, 1.0);
        m = Box::new(Default::default()); // Prevent mass from being free untill its actual owner drops it.
        ode::dMassSetBox(&mut *m, 1.0, 1.0, 1.0, 1.0);
        ode::dBodySetMass(body, &*m);
        ode::dGeomSetBody(geom, body);
        ode::dBodySetPosition(body, location.x, location.y, location.z);
    }

    return (geom, m);
}

fn main() {
    print!("Starting server . . . ");
    let socket = UdpSocket::bind("127.0.0.1:34555").unwrap();

    thread::spawn(move || {
        let mut buf = [0; 100];
        println!("Waiting on data.");
        let (amt, src) = socket.recv_from(&mut buf).unwrap(); //Receive into the buffer
        println!("Recieved {} bytes from {}.", amt, src);

        drop(socket);
    });


    let mut graphix;
    let mut contact_group;
    let mut geoms = Vec::<(dGeomID, Box<dMass>)>::new();
    let world_data;
    let space;
    let world;

    //Init everything
    unsafe {

        graphix = Renderer::init();

        ode::dInitODE();
        world = ode::dWorldCreate();
        space = ode::dHashSpaceCreate(std::ptr::null_mut());
        ode::dWorldSetGravity(world, 0.0, -4.0, 0.0);
        ode::dWorldSetCFM( world, 0.0001);
        ode::dCreatePlane(space, 0.0, 1.0, 0.0, 0.0);
        contact_group = ode::dJointGroupCreate(0);
        world_data = WorldData {world: world, contacts: contact_group};

        for n in 0..100 {
            geoms.push(create_cube(Vec3::new(((n/10)*2) as f32, 3.0, ((n%10)*2) as f32), world, space));
        }
    }
    print!("Done.\n");

    // Do Simulation and rendering
    while !graphix.window.should_close() {
        graphix.window.glfw.poll_events();
        unsafe { // Opengl calls are unsafe
            ode::dSpaceCollide(space, std::mem::transmute(&world_data), near_callback); //Implicit that this function DOESNT change world.
            ode::dWorldQuickStep(world, 0.01);
            ode::dJointGroupEmpty(contact_group);

            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::ClearColor(0.38, 0.906, 0.722, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            for geom_data in geoms.iter() {
                graphix.render_cube(geom_data.0); // Seperate from ODE?
            }
        }

        for(_, event) in glfw::flush_messages(&graphix.events) {
            handle_window_event(&mut graphix.window, event);
        }

        graphix.window.swap_buffers();
    }

    //Do clean ups
    graphix.clean_up();
    unsafe {
        ode::dJointGroupDestroy(contact_group);
        ode::dSpaceDestroy(space);
        ode::dWorldDestroy(world);
        ode::dCloseODE();
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
