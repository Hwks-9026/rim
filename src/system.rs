#![allow(dead_code)]
use rand::prelude::*;
use raylib::{ffi::PI, prelude::Vector3, prelude::Camera3D, prelude::RaylibHandle};
use crate::utils::{self, rotate_vector, vector3_serde};
use serde::{Serialize, Deserialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StarSystemData {
    pub star_mass: f64,
    pub planets: Vec<Planet> 
}


impl StarSystemData {
    pub fn new() -> StarSystemData {
        let mut rng = thread_rng();
        let min_star_mass: f64 = 3.28875 * (10.0 as f64).powi(29);
        let max_star_mass: f64 = 8.77000 * (10.0 as f64).powi(31);
        let star_mass = rng.gen_range(min_star_mass..max_star_mass);
        StarSystemData {
            star_mass,
            planets: generate_planets(rng.gen_range(0..=10))
        }  
    }
    pub fn closest_planet_to_mouse(&self, rl: &mut RaylibHandle, camera: &Camera3D) -> Option<usize> {
        let mouse_pos = rl.get_mouse_position();
        let ray = rl.get_screen_to_world_ray(mouse_pos, camera);

        let mut closest: Option<usize> = None;
        let mut min_distance = f32::MAX;

        for (i, planet) in self.planets.iter().enumerate() {
            let planet_pos = utils::point_on_3d_circle(planet.orbit_normal, planet.orbit_radius as f32, planet.orbit_completion as f32 * 2.0 * PI as f32);
            if crate::map::ray_sphere_intersect(ray.position, ray.direction, planet_pos.scale_by(50.0), 2.0) {
                let dist = (ray.position - planet_pos).length();
                if dist < min_distance {
                    min_distance = dist;
                    closest = Some(i);
                }
            }
        }
        closest 
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Planet {
    pub mass: f64,
    pub orbit_completion: f64, //number from 0 to 1
    pub orbit_radius: f64,
    #[serde(with = "vector3_serde")]
    pub orbit_normal: Vector3,
    pub class: PlanetClass,
    pub moons: Vec<Moon>
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum PlanetClass {
    Terran,
    GasGiant,
    IceGiant,
    Volcanic,
    Desert,
    OceanWorld,
    MetalWorld,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Moon {
    pub moon_type: MoonType,
    pub mass: f64,
    pub orbital_radius: f64,
    #[serde(with = "vector3_serde")]
    pub orbit_normal: Vector3,
    pub orbit_completion: f64
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum MoonType {
    Asteroid,
    RoundDusty,
    SubsurfaceOcean,
}



pub(crate) fn generate_planets(num_planets: usize) -> Vec<Planet> {
    let mut rng = thread_rng();

    // Define orbital order preference by class
    let class_orbit_priority = vec![
        PlanetClass::Volcanic,    // closest
        PlanetClass::MetalWorld,
        PlanetClass::Terran,
        PlanetClass::Desert,
        PlanetClass::OceanWorld,
        PlanetClass::GasGiant,
        PlanetClass::IceGiant,    // farthest
    ];

    // Mass ranges in Earth masses
    let class_mass_range = |class: &PlanetClass| match class {
        PlanetClass::Volcanic => (0.1, 0.5),
        PlanetClass::MetalWorld => (0.1, 1.0),
        PlanetClass::Terran => (0.5, 5.0),
        PlanetClass::Desert => (0.5, 3.0),
        PlanetClass::OceanWorld => (0.8, 6.0),
        PlanetClass::GasGiant => (50.0, 300.0),
        PlanetClass::IceGiant => (10.0, 50.0),
    };

    // Base orbital radius ranges by class (in AU)
    let class_orbit_range = |class: &PlanetClass| match class {
        PlanetClass::Volcanic => (0.1, 0.2),
        PlanetClass::MetalWorld => (0.2, 0.4),
        PlanetClass::Terran => (0.3, 0.5),
        PlanetClass::Desert => (0.2, 0.5),
        PlanetClass::OceanWorld => (0.4, 0.6),
        PlanetClass::GasGiant => (1.0, 1.7),
        PlanetClass::IceGiant => (1.4, 2.0),
    };

    // Moon generation rules
    let moon_types = [MoonType::Asteroid, MoonType::RoundDusty, MoonType::SubsurfaceOcean];
    let moon_mass_range = |mt: &MoonType| match mt {
        MoonType::Asteroid => (0.00001, 0.0001),
        MoonType::RoundDusty => (0.0001, 0.01),
        MoonType::SubsurfaceOcean => (0.005, 0.05),
    };

    // Pick a base inclination offset for the system

    let mut planets = Vec::new();
    let mut used_orbits: Vec<f64> = Vec::new();

    for i in 0..num_planets {
        // Pick class in rough order, but with some randomness
        let class_index = ((i as f64 / num_planets as f64) * class_orbit_priority.len() as f64)
            .round()
            .clamp(0.0, (class_orbit_priority.len() - 1) as f64) as usize;
        let class = *class_orbit_priority.to_owned().clone()
            .get(class_index + rng.gen_range(0..=1).min(class_orbit_priority.len() - 1 - class_index))
            .unwrap_or(&PlanetClass::Terran);

        // Get mass
        let (m_min, m_max) = class_mass_range(&class);
        let mass = rng.gen_range(m_min..m_max);

        // Get orbit radius ensuring no exact duplicates
        let (o_min, o_max) = class_orbit_range(&class);
        let mut orbit_radius;
        loop {
            orbit_radius = rng.gen_range(o_min..o_max);
            if !used_orbits.iter().any(|&o| (o - orbit_radius).abs() < 0.05) {
                used_orbits.push(orbit_radius);
                break;
            }
        }

        // Get orbit inclination within ±20° total spread

        // Generate moons
        let num_moons = match class {
            PlanetClass::GasGiant => rng.gen_range(5..15),
            PlanetClass::IceGiant => rng.gen_range(3..10),
            PlanetClass::Terran | PlanetClass::OceanWorld => rng.gen_range(0..3),
            _ => rng.gen_range(0..2),
        };

        let mut moons = Vec::new();
        let mut used_moon_orbits: Vec<f64> = Vec::new();

        for _ in 0..num_moons {
            let moon_type = &moon_types[rng.gen_range(0..moon_types.len())];
            let (mm_min, mm_max) = moon_mass_range(moon_type);
            let moon_mass = rng.gen_range(mm_min..mm_max);

            let mut moon_orbit;
            loop {
                moon_orbit = rng.gen_range(0.01..0.05); // in AU
                if !used_moon_orbits.iter().any(|&o| (o - moon_orbit).abs() < 0.001) {
                    used_moon_orbits.push(moon_orbit);
                    break;
                }
            }

            moons.push(Moon {
                moon_type: moon_type.to_owned().clone(),
                mass: moon_mass,
                orbital_radius: moon_orbit,
                orbit_completion: rng.gen_range(0..10000) as f64 / 10000.0,
                orbit_normal: random_orbit_normal(&mut rng, 20.0, Vector3::left()),
            });
        }
        let orbit_completion = rng.gen_range(0.0..1.0);
        planets.push(Planet {
            mass,
            orbit_completion,
            orbit_radius,
            orbit_normal: random_orbit_normal(&mut rng, 20.0, Vector3::left()),
            class,
            moons,
        });
    }

    planets.sort_by(|a, b| a.orbit_radius.partial_cmp(&b.orbit_radius).unwrap());
    planets
}


fn random_orbit_normal<R: Rng>(rng: &mut R, max_degrees: f32, base: Vector3) -> Vector3 {
    let max_radians = max_degrees.to_radians();
    // Random tilt axis
    let axis = Vector3::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)).normalized();
    // Random tilt angle
    let tilt = rng.gen_range(-max_radians..max_radians);
    // Rotate base vector
    return rotate_vector(base, axis, tilt).normalized()
}

