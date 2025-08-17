use rand::Rng;
use raylib::prelude::Vector3;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::f32::consts::PI;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
pub(crate) fn random_normalized_vector() -> Vector3 {
    let mut rng = rand::thread_rng();

    // Pick a random point inside a unit sphere
    let x = rng.gen_range(-1.0..=1.0);
    let y = rng.gen_range(-1.0..=1.0);
    let z = rng.gen_range(-1.0..=1.0);
    return Vector3::new(x, y, z).normalized();
}
pub(crate) fn hash_planet_id(id: usize) -> usize {
    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    hasher.finish() as usize
}

pub(crate) fn point_on_3d_circle(normal: Vector3, radius: f32, angle: f32) -> Vector3 {
    let n = normal.normalized();
    let (i, o) = inclination_and_omega(n);
    let pi32 = 3.0 * PI / 2.0;
    let x = radius * ((angle.cos() * (o - pi32).cos()) - (i.cos() * angle.sin() * (o - pi32).sin()));
    let y = radius * ((angle.cos() * (o - pi32).sin()) + (i.cos() * angle.sin() * (o - pi32).cos()));
    let z = radius * angle.sin()*i.sin(); 
    Vector3::new(x, y, z)

}

fn inclination_and_omega(normal: Vector3) -> (f32, f32) {
    let i = normal.z.acos();
    let o = normal.x.atan2(-1.0 * normal.y);
    (i, o) 


}

pub fn rotate_vector(v: Vector3, axis: Vector3, angle: f32) -> Vector3 {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    v * cos_a + axis.cross(v) * sin_a + axis * axis.dot(v) * (1.0 - cos_a)
}

#[derive(Serialize, Deserialize)]
pub struct SerializableVector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Into<Vector3> for SerializableVector3 {
    fn into(self) -> Vector3 {
        Vector3 {
            x: self.x,
            y: self.y,
            z: self.z
        }
    }
}
impl Into<SerializableVector3> for Vector3 {
    fn into(self) -> SerializableVector3 {
        SerializableVector3 {
            x: self.x,
            y: self.y,
            z: self.z
        }
    }
}

pub mod vector3_serde {
    use super::*;


    pub fn serialize<S>(v: &Vector3, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Just forward a tuple of floats
        (v.x, v.y, v.z).serialize(s)
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Vector3, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (x, y, z) = <(f32, f32, f32)>::deserialize(d)?;
        Ok(Vector3 { x, y, z })
    }
}
