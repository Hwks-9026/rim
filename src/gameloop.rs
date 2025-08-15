use std::f64::consts::PI;

use raylib::prelude::*;


use crate::{
    map::{self, Galaxy}, 
    system::{PlanetClass, StarSystemData}, utils::{self, rotate_vector, point_on_3d_circle}
};

pub(crate) fn start_gameloop() {
    let (mut rl, thread) = raylib::init()
        .log_level(TraceLogLevel::LOG_NONE)
        .undecorated()
        .title("Rim")
        .vsync()
        .build();
    rl.set_exit_key(None);
    let galaxy = map::Galaxy::new(200, 15, 250.0, 50.0);
    let stars = get_stars(500, 140.0);
    let game_data = GameData {state: GameState::MapView, galaxy, hovered: None, focused: None, stars };
    gameloop(&mut rl, &thread, game_data);
}


enum GameState {
    MapView,
    StarSystemView,
}


fn gameloop(rl: &mut RaylibHandle, thread: &RaylibThread, mut game_data: GameData) {
    while !rl.window_should_close() {
        match &game_data.state {
            GameState::MapView => {
                gameloop_map_view(rl, thread, &mut game_data);
            }
            GameState::StarSystemView => {
                gameloop_star_system_view(rl, thread, &mut game_data)
            }
        }
    }
}
fn gameloop_star_system_view(rl: &mut RaylibHandle, thread: &RaylibThread, game_data: &mut GameData) {
    let mut camera = Camera3D::orthographic(
        Vector3::new(100.0, 10.0, 100.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        90.0,
    );
    
    let mut goofy_orbits: Vec<Vec<(Vector3, u8)>>;
    let mut orbit_angle = 0.0f32; 
    let orbit_speed = 0.2;        
    let orbit_radius = 150.0;
    while !rl.window_should_close() {
        if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) { break }
        let mut orbit_direction = 0.0;
        if rl.is_key_down(KeyboardKey::KEY_LEFT) {orbit_direction += 1.0}
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {orbit_direction -= 1.0}
        let dt = rl.get_frame_time();
        orbit_angle += orbit_speed * dt * orbit_direction;

        camera.position.x = orbit_radius * orbit_angle.cos();
        camera.position.z = orbit_radius * orbit_angle.sin();
        camera.target = Vector3::zero(); 
        camera.fovy -= 5.0 * rl.get_mouse_wheel_move();
        if camera.fovy > 120.0 {camera.fovy = 120.0};
        if camera.fovy < 20.0 {camera.fovy = 20.0};
        game_data.galaxy.systems[game_data.focused.unwrap()].tick();
        goofy_orbits = {
            let mut orbits = Vec::new();
            for planet in &game_data.galaxy.systems[game_data.focused.unwrap()]
                .system_data.clone().unwrap().planets {
                let mut points = Vec::new();
                let mut i: f64 = 0.0;
                while i < PI * 2.0 {
                    let d = (((i / 2.0 / PI) - planet.orbit_completion + 0.5) % 1.0) - 0.5;
                    let g = (-60.0 * d * d).exp();
                    let mut alpha = (255.0 * g) as u8;
                    if alpha < 10 {alpha = 10}
                    if alpha > 5 {
                        points.push((point_on_3d_circle(planet.orbit_normal, planet.orbit_radius as f32, i as f32), alpha));
                    }
                    i += 0.001;
                }
                orbits.push(points);
            }
            orbits
        };

        draw_star_system_view(rl, thread, &camera, game_data, &goofy_orbits);
         
    }


    game_data.hovered = None;
    game_data.focused = None;
    game_data.state = GameState::MapView;
}

fn draw_star_system_view(
    rl: &mut RaylibHandle, 
    thread: &RaylibThread, 
    camera: &Camera3D, 
    game_data: &mut GameData,
    orbits: &Vec<Vec<(Vector3, u8)>>
    ) {
    let mut d = rl.begin_drawing(thread);
    
    //It's okay to unwrap these things because the only way to get to star_system_view is by having
    //both these options set.
    let sys_data = &game_data.galaxy.systems[game_data.focused.unwrap()].system_data.clone().unwrap();

    d.clear_background(Color::BLACK);
    {
        let mut d3 = d.begin_mode3D(camera);
        for star in &game_data.stars {
            d3.draw_point3D(star, Color::WHITE);
        }
        d3.draw_sphere(Vector3::zero(), sys_data.star_mass.log10() as f32 / 20.0, Color::YELLOW);
        
        for (point, alpha) in orbits.iter().flatten() {
            d3.draw_point3D(point.scale_by(50.0), Color::new(255, 255, 255, *alpha));
        }

        for planet in &sys_data.planets {
            
            
            let display_radius = planet.orbit_radius as f32 * 50.0;
            let angle = planet.orbit_completion as f32 * 2.0 * PI as f32;
            //d3.draw_circle_3D(Vector3::zero(), display_radius, planet.orbit_normal, 80.0, );

            // Parametric circle position in tilted plane
            
            let planet_pos = utils::point_on_3d_circle(planet.orbit_normal, display_radius, angle);

            // Planet color
            let planet_color = match planet.class {
                PlanetClass::Volcanic => Color::ORANGE,
                PlanetClass::MetalWorld => Color::GRAY,
                PlanetClass::Terran => Color::GREEN,
                PlanetClass::Desert => Color::BEIGE,
                PlanetClass::OceanWorld => Color::BLUE,
                PlanetClass::GasGiant => Color::PURPLE,
                PlanetClass::IceGiant => Color::SKYBLUE,
            };
            d3.draw_sphere(planet_pos, 0.5, planet_color); 
        }

    }
}


fn gameloop_map_view(rl: &mut RaylibHandle, thread: &RaylibThread, mut game_data: &mut GameData) {
    let mut camera = Camera3D::orthographic(
        Vector3::new(0.0, 0.0, 150.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        120.0,
    );
    let mut orbit_angle = 0.0f32; 
    let orbit_speed = 0.1;        
    let orbit_radius = 150.0;
    let mut selecting = false;
    let mut fully_zoomed_frames = 0;
    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        orbit_angle += orbit_speed * dt;

        camera.position.x = orbit_radius * orbit_angle.cos();
        camera.position.z = orbit_radius * orbit_angle.sin();
        match &game_data.focused {
            None => { camera.target = Vector3::zero(); },
            Some(focus) => {camera.target = game_data.galaxy.systems[*focus].position;}
        }
        game_data.galaxy.wiggle(dt);

        game_data.hovered = game_data.galaxy.closest_system_to_mouse(rl, &camera);
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            match game_data.hovered {
                None => {},
                Some(i) => {
                    match &game_data.galaxy.systems[i].system_data {
                        Some(_) => {}
                        None => {game_data.galaxy.systems[i].system_data = Some(StarSystemData::new())}
                    }
                    game_data.focused = Some(i);
                    camera.fovy = 50.0; 
                }
            }
        }
        if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            match game_data.focused {
                None => {},
                Some(_) => {
                    game_data.focused = None; 
                    camera.fovy = 120.0; 
                    selecting = false;
                    fully_zoomed_frames = 0;
                }
            }
        }
        if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
            match game_data.focused {
                None => {},
                Some(_) => {
                    selecting = true;
                }
            }
        }
        if selecting {
            if camera.fovy > 5.0 {camera.fovy -= 0.5}
            else {
                if fully_zoomed_frames < 10 {
                    fully_zoomed_frames += 1;
                }
                else {
                    //clean up and switch to solar system view
                    game_data.state = GameState::StarSystemView;
                    break 
                }
            }
        }
        draw_map_view(rl, thread, &camera, &mut game_data);
    }
}





fn draw_map_view(rl: &mut RaylibHandle, thread: &RaylibThread, camera: &Camera3D, game_data: &mut GameData) {

    let mut d = rl.begin_drawing(thread);
    d.clear_background(Color::BLACK);
    {
        let mut d3 = d.begin_mode3D(camera);
        for star in &game_data.stars {
            d3.draw_point3D(star, Color::WHITE);
        }

        for (i, system) in game_data.galaxy.systems.iter().enumerate() {
            let (size, mut color) = match game_data.hovered {
                Some(val) => {
                    if val == i { (0.9, Color::POWDERBLUE) }
                    else { (0.5, Color::new(55, 55, 55, 255)) }
                },
                None => { 
                    if let Some(_) = system.system_data {
                        (0.6, Color::new(80, 80, 110, 255)) 
                    }
                    else {
                        (0.5, Color::new(90, 90, 90, 255)) 
                    }
                }
            };
            match game_data.focused {
                None => {}
                Some(focus) => {
                    if i == focus {color = Color::YELLOW}
                }
            }
            d3.draw_sphere(system.position, size, color.alpha(camera.fovy / 50.0));
            d3.draw_sphere(system.position, size + 0.1, color.alpha(0.5));

            // Draw connections
            for &conn_idx in &system.connections {
                if conn_idx > i {
                    let conn = &game_data.galaxy.systems[conn_idx];
                    d3.draw_line_3D(
                        system.position,
                        conn.position,
                        Color::new(255, 255, 255, 20),
                    );
                }
            }
        }

    }
    /*
    match game_data.hovered {
        None => {
            let string = match game_data.focused {
                Some(focus) => {
                    game_data.galaxy.systems[focus].get_hover_string()
                },
                None => {"Not Hovering Over A Star System".to_string()}
            };
            d.draw_text(&string, 10, 50, 20, Color::new(200, 200, 200, (camera.fovy * 5.0) as u8))
        },
        Some(i) => {
            d.draw_text(&game_data.galaxy.systems[i].get_hover_string(), 10, 50, 20, Color::new(200, 200, 200, (camera.fovy * 5.0) as u8))
        }
    }
    */
    if camera.fovy < 60.0 {
        d.draw_text("Press Return To View The Selected Star System.\nPress Escape To Go Back to Map View", 10, 10, 20, Color::new(100, 100, 200, (camera.fovy * 5.0) as u8));
    }

}



struct GameData {
    state: GameState,
    galaxy: Galaxy,
    hovered: Option<usize>,
    focused: Option<usize>,
    stars: Vec<Vector3>
}

fn get_stars(num_stars: usize, starfield_radius: f32) -> Vec<Vector3> {

    let mut stars = Vec::new();

    for _ in 0..num_stars {
        let theta = rand::random::<f32>() * std::f32::consts::TAU;
        let phi = (rand::random::<f32>() - 0.5) * std::f32::consts::PI;
        let r = starfield_radius;
        let x = r * phi.cos() * theta.cos();
        let y = r * phi.sin();
        let z = r * phi.cos() * theta.sin();
        stars.push(Vector3::new(x, y, z));
    }
    stars
}

