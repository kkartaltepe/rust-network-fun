extern crate ode;
extern crate libc;

use std;
use ode::*;
use vec::Vec3;



extern fn near_callback(data : *mut libc::c_void, obj1 : ode::dGeomID, obj2 : ode::dGeomID) {
    unsafe {
        let data: &mut (dWorldID, dJointGroupID) = std::mem::transmute(data);
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
            let joint = ode::dJointCreateContact(data.0, data.1, &mut contact);
            ode::dJointAttach(joint, b1, b2);
        }
    }
}

pub struct Simulation {
    world: dWorldID,
    space: dSpaceID,
    contact_group: dJointGroupID,
    pub geoms: Vec<(dGeomID, Box<dMass>)>,
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
        ode::dWorldSetGravity(world, 0.0, -4.0, 0.0);
        ode::dWorldSetCFM(world, 0.0001);
        ode::dCreatePlane(space, 0.0, 1.0, 0.0, 0.0);
        contact_group = ode::dJointGroupCreate(0);
        }

        return Simulation{
            world: world,
            space: space,
            contact_group: contact_group,
            geoms: Vec::new()
        };
    }


    pub fn step(&mut self) {
        unsafe {
        ode::dSpaceCollide(self.space, std::mem::transmute(&mut (self.world, self.contact_group)), near_callback); //Implicit that this function DOESNT change world.
        ode::dWorldQuickStep(self.world, 0.01);
        ode::dJointGroupEmpty(self.contact_group);
        }
    }

    pub fn create_cube(&mut self, location: Vec3) {
        let body;
        let geom;
        let mut m: Box<ode::dMass>;
        unsafe {
            body = ode::dBodyCreate(self.world);
            geom = dCreateBox(self.space, 1.0, 1.0, 1.0);
            m = Box::new(Default::default()); // Prevent mass from being free untill its actual owner drops it.
            ode::dMassSetBox(&mut *m, 1.0, 1.0, 1.0, 1.0);
            ode::dBodySetMass(body, &*m);
            ode::dGeomSetBody(geom, body);
            ode::dBodySetPosition(body, location.x, location.y, location.z);
        }

        self.geoms.push((geom, m));
    }

    pub fn clean_up(&mut self) {
        unsafe {
        ode::dJointGroupDestroy(self.contact_group);
        ode::dSpaceDestroy(self.space);
        ode::dWorldDestroy(self.world);
        ode::dCloseODE();
        }
    }

}
