#![allow(unused_imports)]
use std::ops::{Add, Sub, Neg};
use std::f32::*;

#[derive(Copy, Clone)]
pub struct Vec3 {
    pub x : f32,
    pub y : f32,
    pub z : f32
}

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, other: Vec3) -> Vec3 {
     return Vec3 {
         x: self.x + other.x,
         y: self.y + other.y,
         z: self.z + other.z
     }
    }
}

impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Vec3 {
     return Vec3 {
         x: -self.x,
         y: -self.y,
         z: -self.z
     }
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, other: Vec3) -> Vec3 {
     return Vec3 {
         x: self.x - other.x,
         y: self.y - other.y,
         z: self.z - other.z
     }
    }
}

impl Vec3 {
    pub fn new(x_: f32, y_: f32, z_: f32) -> Vec3 {
        return Vec3 {
            x: x_,
            y: y_,
            z: z_
        }
    }

    pub fn normalize(mut self) -> Vec3 {
        let len = self.dot(self).sqrt();
        self.x = self.x/len;
        self.y = self.y/len;
        self.z = self.z/len;
        self
    }

    pub fn cross(self, b : Vec3) -> Vec3 {
        return Vec3 {
            x: self.y*b.z - self.z*b.y,
            y: self.z*b.x - self.x*b.z,
            z: self.x*b.y - self.y*b.x
        }
    }

    pub fn dot(self, other : Vec3) -> f32 {
     return (self.x * other.x)
            + (self.y * other.y)
            + (self.z * other.z)
    }
}
