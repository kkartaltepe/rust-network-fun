#![allow(dead_code)]

extern crate ode;
extern crate libc;
extern crate byteorder;

use std;
use ode::*;
use vec::Vec3;

use std::io::Cursor;
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};



extern fn near_callback(data : *mut libc::c_void, obj1 : ode::dGeomID, obj2 : ode::dGeomID) {
    unsafe {
        let data: &mut (dWorldID, dJointGroupID) = std::mem::transmute(data);
        let b1 = dGeomGetBody(obj1);
        let b2 = dGeomGetBody(obj2);
        let mut contact = ode::dContact{..Default::default()};
        contact.surface.mode = ode::dContactApprox1 as i32;
        contact.surface.mu = ode::dInfinity;
        if ode::dCollide(obj1, obj2, 1, &mut contact.geom, std::mem::size_of::<ode::dContact>() as libc::c_int) != 0 {
            //println!("Collision detected!");
            let joint = ode::dJointCreateContact(data.0, data.1, &mut contact);
            ode::dJointAttach(joint, b1, b2);
        }
    }
}

fn fmt_3(e: [f32; 3]) -> String {
    format!("({},{},{})", e[0], e[1], e[2])
}

fn fmt_12(e: [f32; 12]) -> String {
    format!("({},{},{}),({},{},{}),({},{},{}),({},{},{})",
            e[0], e[1], e[2], e[3], e[4], e[5], e[6], e[7], e[8], e[9], e[10], e[11])
}

pub struct Simulation {
    world: dWorldID,
    space: dSpaceID,
    contact_group: dJointGroupID,
    pub geoms: Vec<(dGeomID, Box<dMass>)>,
    paused: bool,
}

impl Simulation {
    pub fn init() -> Simulation {
        let world;
        let space;
        let contact_group;

        unsafe {
        ode::dInitODE();
        world = ode::dWorldCreate();
        space = ode::dHashSpaceCreate(std::ptr::null_mut());
        ode::dWorldSetGravity(world, 0.0, -10.0, 0.0);
        ode::dWorldSetCFM(world, 0.0001);
        ode::dCreatePlane(space, 0.0, 1.0, 0.0, 0.0);
        contact_group = ode::dJointGroupCreate(0);
        }

        return Simulation{
            world: world,
            space: space,
            contact_group: contact_group,
            geoms: Vec::new(),
            paused: true,
        };
    }


    pub fn step(&mut self) {
        if self.paused {
            return
        }
        unsafe {
        ode::dSpaceCollide(self.space, std::mem::transmute(&mut (self.world, self.contact_group)), near_callback); //Implicit that this function DOESNT change world.
        ode::dWorldQuickStep(self.world, 0.01);
        ode::dJointGroupEmpty(self.contact_group);
        }
    }

    pub fn create_cube(&mut self, mass: f32, location: Vec3) {
        let body;
        let geom;
        let mut m: Box<ode::dMass>;
        unsafe {
            body = ode::dBodyCreate(self.world);
            geom = dCreateBox(self.space, 1.0, 1.0, 1.0);
            m = Box::new(Default::default()); // Prevent mass from being free untill its actual owner drops it.
            ode::dMassSetBox(&mut *m, mass, 1.0, 1.0, 1.0);
            ode::dBodySetMass(body, &*m);
            ode::dGeomSetBody(geom, body);
            ode::dBodySetPosition(body, location.x, location.y, location.z);
        }

        self.geoms.push((geom, m));
        println!("Created cube #{}", self.geoms.len());
    }

    pub fn apply_force(&self, geom: dGeomID, force: Vec3) {
        unsafe {
        let body = ode::dGeomGetBody(geom);
        ode::dBodyAddForce(body, force.x, force.y, force.z);
        }
    }

    pub fn get_location(&self, geom: dGeomID) -> Vec3 {
        unsafe {
        let ppos = ode::dGeomGetPosition(geom);
        let vec = std::slice::from_raw_parts(ppos, 3);
        return Vec3 { x: vec[0], y: vec[1], z: vec[2] }
        }
    }

    pub fn clean_up(&mut self) {
        unsafe {
        ode::dJointGroupDestroy(self.contact_group);
        ode::dSpaceDestroy(self.space);
        ode::dWorldDestroy(self.world);
        ode::dCloseODE();
        }
    }

    //TODO: Move everything to quaternions
    pub fn serialize(&self, init: bool) -> Vec<u8> {
        let mut buf = vec![];
        buf.write_u8(self.paused as u8).unwrap();
        buf.write_u8(init as u8).unwrap();
        if !self.paused || init {
            buf.write_u32::<LittleEndian>(self.geoms.len() as u32).unwrap();
            //let mut print = true;
            for &(geom, ref m) in self.geoms.iter() {
                let pos;
                let rot;
                let vel;
                unsafe {
                pos = std::slice::from_raw_parts(ode::dGeomGetPosition(geom), 3);
                rot = std::slice::from_raw_parts(ode::dGeomGetRotation(geom), 12); // 3 rows, 4 columns.
                let body = dGeomGetBody(geom);
                vel = std::slice::from_raw_parts(ode::dBodyGetLinearVel(body), 3);
                //if print {
                    //println!("Vel: {}, {}, {}", vel[0], vel[1], vel[2]);
                    //print = false;
                //}
                }
                if vel[0].abs() <= 0.1f32 &&
                   vel[1].abs() <= 0.1f32 &&
                   vel[2].abs() <= 0.1f32 &&
                   !init {
                    buf.write_u8(0).unwrap(); // Cube at rest, probably fine to not send.
                } else {
                    buf.write_u8(1).unwrap(); // Cube in motion more data to follow.
                    for p in 0..3 {
                        buf.write_f32::<LittleEndian>(pos[p]).unwrap();
                    }
                    for r in 0..12 {
                        buf.write_f32::<LittleEndian>(rot[r]).unwrap();
                    }
                    if init { // If we are sending an initalization packet contain some extra info.
                        buf.write_f32::<LittleEndian>(m.mass).unwrap();
                    }
                }
            }
        }
        //println!("Serialized state into {} bytes", buf.len());
        return buf;
    }

    pub fn deserialize(&mut self, buf: &[u8]) {
        let mut input = Cursor::new(buf);
        let is_paused = input.read_u8().unwrap() != 0u8;
        let is_init = input.read_u8().unwrap() != 0u8;

        self.paused = is_paused;
        if !self.paused || is_init {
            let num_geoms = input.read_u32::<LittleEndian>().unwrap() as usize;
            //println!("Decoding {} geoms",num_geoms);
            for i in 0..num_geoms{
                if input.read_u8().unwrap() == 0 {
                    continue; //This cube has no velocity and it is not an init frame
                              // so no data for it follows.
                }

                let mut pos = [0f32; 3];
                let mut rot = [0f32; 12];
                for p in 0..3 {
                    pos[p] = input.read_f32::<LittleEndian>().unwrap();
                }
                for r in 0..12 {
                    rot[r] = input.read_f32::<LittleEndian>().unwrap();
                }
                let mut mass = 1.0;
                if is_init {
                    mass = input.read_f32::<LittleEndian>().unwrap();
                }
                if i == self.geoms.len() {
                    self.create_cube(mass, Vec3::new(pos[0], pos[1], pos[2]));
                }
                unsafe {
                    dGeomSetPosition(self.geoms[i].0, pos[0], pos[1], pos[2]);
                    dGeomSetRotation(self.geoms[i].0, &rot);
                }
            }
        }
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }


}
