#![allow(dead_code)]
use crate::utils;
use crate::utils::*;
use crate::system::*;
use rand::Rng;
use raylib::prelude::*;
use std::f64::consts::PI;
use std::fmt::format;

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StarSystem {
    #[serde(with = "vector3_serde")]
    pub position: Vector3,
    #[serde(with = "vector3_serde")]
    origin: Vector3,
    #[serde(with = "vector3_serde")]
    pub drift_direction: Vector3,
    pub connections: Vec<usize>,
    pub name: usize,
    pub system_data: Option<StarSystemData>,
    pub explored: bool
}

const SPRING_STRENGTH: f32 = 0.001;
const DAMPING: f32 = 0.95;

impl StarSystem {
    fn drift(&mut self, dt: f32) {
        self.drift_direction += random_normalized_vector().scale_by(0.003);
        let o = self.origin - self.position;
        self.drift_direction += o.scale_by(SPRING_STRENGTH);
        self.drift_direction.scale(DAMPING);

        self.position += self.drift_direction.scale_by(dt * 60.0);
    }
    pub fn get_hover_string(&self) -> String {
        match &self.system_data {
            None => {
                format!("No Data Available For System {:X}.\nSelect System to Scan.", self.name)
            }
            Some(data) => {
                let mut hover_string: String = format!("System {:X}:\n", self.name);
                hover_string += format!("Number of Planets: {}\n", data.planets.len()).as_str();
                for (i, planet) in data.planets.iter().enumerate() {
                    hover_string += "---\n";
                    hover_string += format!("Planet {:X}-{}\n", self.name, 
                        utils::num_to_letter(i as u8).unwrap().to_ascii_uppercase()).as_str();
                    hover_string += format!("   {:?} Planet.\n", planet.class).as_str();
                    hover_string += format!("   Orbital radius: {:.3} std.\n", planet.orbit_radius).as_str();
                    hover_string += format!("   {} Moons.\n", planet.moons.len()).as_str();
                }
                hover_string.to_string()
            }

        }
    }
    pub fn tick(&mut self) {
        match &mut self.system_data {
            None => {
            }
            Some(data) => {
                for planet in &mut data.planets {
                    planet.orbit_completion += 0.0002 / planet.orbit_radius;
                    if planet.orbit_completion > 1.0 {
                        planet.orbit_completion = 0.0;
                    }
                    for moon in &mut planet.moons {

                        moon.orbit_completion += 0.0002 / moon.orbital_radius;
                        if moon.orbit_completion > 1.0 {
                            moon.orbit_completion = 0.0;
                        }
                    }
                } 
            }

        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Galaxy {
    pub systems: Vec<StarSystem>,
}
impl Galaxy {
    pub fn new(
        num_systems: usize,
        connections_per_system: usize,
        amplitude: f64,
        radius: f64,
    ) -> Galaxy {
        let mut rng = rand::thread_rng();
        let mut systems = Vec::with_capacity(num_systems);

        // Step 1: Generate points on a sphere (Fibonacci sphere)
        let offset = 2.0 / num_systems as f64;
        let increment = PI * (3.0 - (5.0f64).sqrt());

        for i in 0..num_systems {
            let y = ((i as f64) * offset) - 1.0 + (offset / 2.0);
            let r = (1.0 - y * y).sqrt();
            let phi = ((i as f64) % num_systems as f64) * increment;

            let mut x = phi.cos() * r;
            let mut z = phi.sin() * r;
            let mut y = y;

            // Step 2: Apply random offsets
            x += rng.gen_range(-amplitude..amplitude);
            y += rng.gen_range(-amplitude..amplitude);
            z += rng.gen_range(-amplitude..amplitude);

            // Normalize back to sphere surface
            let length = (x * x + y * y + z * z).sqrt();
            x = (x / length) * radius;
            y = (y / length) * radius;
            z = (z / length) * radius;

            systems.push(StarSystem {
                position: Vector3::new(x as f32, y as f32, z as f32),
                origin: Vector3::new(x as f32, y as f32, z as f32),
                drift_direction: Vector3::zero(),
                connections: Vec::new(),
                system_data: Some(crate::file_generator::generate_system_data()),
                name: crate::utils::hash_planet_id(i) as u32 as usize,
                explored: false
            });
        }

        // Step 3: Connect systems to nearest neighbors
        for i in 0..num_systems {
            // Sort other systems by distance to this one
            let mut distances: Vec<(usize, f32)> = systems
                .iter()
                .enumerate()
                .filter(|&(j, _)| j != i)
                .map(|(j, sys)| {
                    let dx = sys.position.x - systems[i].position.x;
                    let dy = sys.position.y - systems[i].position.y;
                    let dz = sys.position.z - systems[i].position.z;
                    (j, (dx * dx + dy * dy + dz * dz).sqrt())
                })
                .collect();

            distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            for &(j, _) in distances.iter().take(connections_per_system) {
                if !systems[i].connections.contains(&j) {
                    systems[i].connections.push(j);
                }
                if !systems[j].connections.contains(&i) {
                    systems[j].connections.push(i);
                }
            }
        }
        Galaxy { systems }
    }
    pub fn wiggle(&mut self, dt: f32) {
        for sys in self.systems.iter_mut() {
            sys.drift(dt)
        }
    }
    pub fn closest_system_to_mouse(&self, rl: &mut RaylibHandle, camera: &Camera3D) -> Option<usize> {
        let mouse_pos = rl.get_mouse_position();
        let ray = rl.get_screen_to_world_ray(mouse_pos, camera);

        let mut closest: Option<usize> = None;
        let mut min_distance = f32::MAX;

        for (i, system) in self.systems.iter().enumerate() {
            if ray_sphere_intersect(ray.position, ray.direction, system.position, 2.0) {
                let dist = (ray.position - system.position).length();
                if dist < min_distance {
                    min_distance = dist;
                    closest = Some(i);
                }
            }
        }
        closest 
    }
}


fn ray_sphere_intersect(ray_pos: Vector3, ray_dir: Vector3, sphere_pos: Vector3, sphere_radius: f32) -> bool {
    let l = Vector3 {
        x: sphere_pos.x - ray_pos.x,
        y: sphere_pos.y - ray_pos.y,
        z: sphere_pos.z - ray_pos.z,
    };

    let tca = l.x*ray_dir.x + l.y*ray_dir.y + l.z*ray_dir.z;
    if tca < 0.0 { return false; }

    let d2 = l.x*l.x + l.y*l.y + l.z*l.z - tca*tca;
    d2 <= sphere_radius * sphere_radius
}
