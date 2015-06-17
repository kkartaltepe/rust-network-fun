#![feature(convert)]
extern crate glfw;
extern crate gl;
extern crate libc;
extern crate ode;

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
use ode::*;

//static VERTEX_DATA : [f32; 9] = [
    //-1.0, -1.0, -1.0,
    //1.0, -1.0, -1.0,
    //0.0, 1.0, -1.0
//];

struct WorldData {
    world : ode::dWorldID,
    contacts : ode::dJointGroupID,
}

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

const IDENT_MAT : [f32; 16] = [
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0
];

static mut VIEW_MAT : [f32; 16] = IDENT_MAT;

static mut PROJ_MAT : [f32; 16] = IDENT_MAT;

static mut MODEL_MAT : [f32; 16] = IDENT_MAT;


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

fn render_cube(geom: dGeomID, program_id: GLuint, vertexbuffer: GLuint, normalbuffer: GLuint) {
    unsafe {
    let rotMat: *const f32 = ode::dGeomGetRotation(geom);
    let posVec: *const f32 = ode::dGeomGetPosition(geom);
    let rustPos = std::slice::from_raw_parts(posVec, 3);
    let rustRot = std::slice::from_raw_parts(rotMat, 16);
    for i in 0..12 {
        MODEL_MAT[i] = rustRot[i];
    }
    translate(&mut MODEL_MAT, rustPos[0], rustPos[1], rustPos[2]);
    //println!("Rot[] = [");
    //for i in 0..16 {
        //print!("{}, ", rustRot[i]);
        //if (i+1)%4 == 0 {
            //print!("\n");
        //}
    //}
    //println!("]");
    let model_mat_id = gl::GetUniformLocation(program_id, CString::new("model").unwrap().as_ptr());
    gl::UniformMatrix4fv(model_mat_id, 1, gl::TRUE, &MODEL_MAT[0]);

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
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(WindowHint::ContextVersion(3, 2));
    let (mut window, events) = glfw.create_window(600, 600, "Server Window", glfw::WindowMode::Windowed)
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
    let mut contact_group;
    let mut geoms = Vec::<(dGeomID, Box<dMass>)>::new();
    let world_data;
    let space;
    let world;

    let view_id;
    //Init everything
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

        perspective(&mut PROJ_MAT, 0.1, 400.0, 1.0, 1.0);
        look_at(&mut VIEW_MAT,
                Vec3::new(0.0, 0.0, -1.0),
                Vec3::new(0.0, 3.0, -7.0),
                Vec3::new(0.0, 1.0, 0.0)
                );

        view_id = gl::GetUniformLocation(program_id, CString::new("view").unwrap().as_ptr());
        let proj_id = gl::GetUniformLocation(program_id, CString::new("proj").unwrap().as_ptr());

        gl::UniformMatrix4fv(view_id, 1, gl::FALSE, &VIEW_MAT[0]);
        gl::UniformMatrix4fv(proj_id, 1, gl::TRUE, &PROJ_MAT[0]);

        ode::dInitODE();
        world = ode::dWorldCreate();
        space = ode::dHashSpaceCreate(std::ptr::null_mut());
        ode::dWorldSetGravity(world, 0.0, -4.0, 0.0);
        ode::dWorldSetCFM( world, 0.0001);
        ode::dCreatePlane(space, 0.0, 1.0, 0.0, 0.0);
        contact_group = ode::dJointGroupCreate(0);
        world_data = WorldData {world: world, contacts: contact_group};

        for n in 0..100 {
            geoms.push(create_cube(Vec3::new(((n/10)*2) as f32, 0.0, ((n%10)*2) as f32), world, space));
        }
    }
    print!("Done.\n");

    // Do Simulation and rendering
    while !window.should_close() {
        glfw.poll_events();
        unsafe { // Opengl calls are unsafe
            ode::dSpaceCollide(space, std::mem::transmute(&world_data), near_callback); //Implicit that this function DOESNT change world.
            ode::dWorldQuickStep(world, 0.01);
            ode::dJointGroupEmpty(contact_group);

            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::ClearColor(0.38, 0.906, 0.722, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            for geom_data in geoms.iter() {
                render_cube(geom_data.0, program_id, vertexbuffer, normalbuffer);
            }
            //render_cube(geom_mass.0, program_id, vertexbuffer, normalbuffer);
            //render_cube(geom_mass2.0, program_id, vertexbuffer, normalbuffer);
        }

        for(_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event);
        }

        window.swap_buffers();
    }

    //Do clean ups
    unsafe {
        gl::DeleteProgram(program_id);
        gl::DeleteShader(vert_shader);
        gl::DeleteShader(frag_shader);
        gl::DeleteBuffers(1, &vertexbuffer);
        gl::DeleteVertexArrays(1, &vert_array_id);

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
