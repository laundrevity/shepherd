// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(extract_if)]

use rand::{thread_rng, Rng};
use serde::Serialize;
use std::collections::HashMap;
use tauri::{async_runtime::RwLock, State, Window};
use tokio::time::{sleep, Duration};

const WINDOW_WIDTH: f64 = 1200.0;
const WINDOW_HEIGHT: f64 = 800.0;

const PLAYER_SPEED: f64 = 5.0;
const ENEMY_SPEED: f64 = 4.0;

const TICK_CYCLE_MS: u64 = 16;
const ENEMY_SPAWN_INTERVAL: u64 = 5000;
const GATE_SPAWN_INTERVAL: u64 = 10000;

const CIRCLE_RADIUS: f64 = 20.0;
const DIAMOND_RADIUS: f64 = 20.0;
const TRIANGLE_RADIUS: f64 = 50.0;
const SQUARE_RADIUS: f64 = 5.0;

const BOOM_RADIUS: f64 = 125.0;
const GATE_BUFFER: f64 = 50.0;

#[derive(Clone, Debug, Serialize)]
enum Sprite {
    Triangle(f64, f64, f64), // x coordinate, y coordinate, rotation
    Circle(f64, f64),
    Diamond(f64, f64),
    Square(f64, f64),
    Point(f64, f64),
}

impl Sprite {
    fn get_coords(&self) -> (f64, f64) {
        match self {
            Sprite::Triangle(x, y, _) => (*x, *y),
            Sprite::Circle(x, y) => (*x, *y),
            Sprite::Diamond(x, y) => (*x, *y),
            Sprite::Square(x, y) => (*x, *y),
            _ => (0.0, 0.0),
        }
    }

    fn get_vertices(&self) -> Vec<(f64, f64)> {
        let mut vertices = Vec::new();

        match self {
            Sprite::Triangle(x, y, rotation) => {
                for i in 0..3 {
                    let angle = 2.0 * std::f64::consts::PI / 3.0 * i as f64 + rotation.to_radians();
                    vertices.push((
                        x + TRIANGLE_RADIUS * angle.cos(),
                        y + TRIANGLE_RADIUS * angle.sin(),
                    ));
                }
            }
            Sprite::Diamond(x, y) => {
                vertices.push((*x, y - DIAMOND_RADIUS));
                vertices.push((x + DIAMOND_RADIUS, *y));
                vertices.push((*x, y + DIAMOND_RADIUS));
                vertices.push((x - DIAMOND_RADIUS, *y));
            }
            Sprite::Square(x, y) => {
                // Multiplier has radius 5 in backend so R^2 + R^2 = S^2 => S/2 = sqrt(2) * R / 2  = 7.07/2 = 3.53
                let s = SQUARE_RADIUS / 2f64.sqrt();
                vertices.push((x - s, y - s));
                vertices.push((x - s, y + s));
                vertices.push((x + s, y - s));
                vertices.push((x + s, y + s));
            }
            _ => {}
        }

        vertices
    }
}

#[derive(Clone, Debug)]
struct Game {
    player: Sprite,
    triangles: Vec<Sprite>,
    diamonds: Vec<Sprite>,
    multipliers: Vec<Sprite>,
    score: u64,
    multiplier: u64,
    key_states: HashMap<String, bool>,
    pending_boom_locations: Vec<(f64, f64)>,
    paused: bool,
    spawn_count: usize,
}

struct AppState {
    game: RwLock<Game>,
}

impl Game {
    fn new() -> Self {
        Self {
            player: Sprite::Circle(200.0, 200.0),
            triangles: vec![Sprite::Triangle(100.0, 100.0, 0.0)],
            diamonds: Vec::new(),
            multipliers: Vec::new(),
            score: 0,
            multiplier: 1,
            key_states: HashMap::new(),
            pending_boom_locations: Vec::new(),
            paused: false,
            spawn_count: 1,
        }
    }

    fn boom(&mut self, bx: f64, by: f64) {
        // TODO: send an explosion animation to the frontend
        let boomed_diamonds = self
            .diamonds
            .extract_if(|diamond| {
                let (dx, dy) = diamond.get_coords();
                // Keep the diamonds too far from the explosion
                ((dx - bx).powi(2) + (dy - by).powi(2)).sqrt() < BOOM_RADIUS
            })
            .collect::<Vec<Sprite>>();

        for diamond in boomed_diamonds {
            let (dx, dy) = diamond.get_coords();
            // increment score and create a multiplier
            self.score += self.multiplier;
            self.multipliers.push(Sprite::Square(dx, dy));
        }
    }

    fn get_triangle_vertices(&self, x: f64, y: f64, rotation: f64) -> Vec<(f64, f64)> {
        // R = s / sqrt(3)
        // R: radius of circumscribing circle
        // s: side of circumscribed triangle
        let mut vertices = Vec::new();

        for i in 0..3 {
            let angle = 2.0 * std::f64::consts::PI / 3.0 * i as f64 + rotation.to_radians();
            vertices.push((
                x + TRIANGLE_RADIUS * angle.cos(),
                y + TRIANGLE_RADIUS * angle.sin(),
            ));
        }

        vertices
    }

    fn get_sprites(&self) -> Vec<Sprite> {
        let mut sprites = vec![self.player.clone()];
        sprites.extend(self.triangles.clone());
        sprites.extend(self.diamonds.clone());
        sprites.extend(self.multipliers.clone());
        sprites
    }

    fn tick(&mut self) {
        if !self.paused {
            self.rotate_triangles();
            self.move_player();
            self.move_enemies();

            self.check_collisions();
        }
    }

    fn rotate_triangles(&mut self) {
        for triangle in self.triangles.iter_mut() {
            if let Sprite::Triangle(_, _, ref mut rot) = triangle {
                *rot += 1.0;
            }
        }
    }

    fn move_player(&mut self) {
        if let Sprite::Circle(ref mut x, ref mut y) = self.player {
            let mut dx = 0.0;
            let mut dy = 0.0;

            if *self.key_states.get("w").unwrap_or(&false) {
                dy -= PLAYER_SPEED as f64;
            }
            if *self.key_states.get("a").unwrap_or(&false) {
                dx -= PLAYER_SPEED as f64;
            }
            if *self.key_states.get("s").unwrap_or(&false) {
                dy += PLAYER_SPEED as f64;
            }
            if *self.key_states.get("d").unwrap_or(&false) {
                dx += PLAYER_SPEED as f64;
            }

            // Scale speed for diagonal movement
            let speed_scale = if dx != 0.0 && dy != 0.0 {
                (2f64).sqrt() / 2.0
            } else {
                1.0
            };

            *x = (*x + dx * speed_scale)
                .max(20.0)
                .min(WINDOW_WIDTH as f64 - 20.0);
            *y = (*y + dy * speed_scale)
                .max(20.0)
                .min(WINDOW_HEIGHT as f64 - 20.0);
        }
    }

    fn spawn_enemy(&mut self) {
        let mut rng = thread_rng();
        let (x, y) = match rng.gen_range(0..4) {
            0 => (0.0, 0.0),
            1 => (WINDOW_WIDTH, 0.0),
            2 => (0.0, WINDOW_HEIGHT),
            _ => (WINDOW_WIDTH, WINDOW_HEIGHT),
        };

        self.diamonds.push(Sprite::Diamond(x, y));
    }

    fn spawn_gate(&mut self) {
        let mut rng = thread_rng();
        let gx = rng.gen_range(GATE_BUFFER..(WINDOW_WIDTH - GATE_BUFFER));
        let gy = rng.gen_range(GATE_BUFFER..(WINDOW_HEIGHT - GATE_BUFFER));
        let gr = rng.gen_range(0.0..360.0);
        self.triangles.push(Sprite::Triangle(gx, gy, gr))
    }

    fn move_enemies(&mut self) {
        let (player_x, player_y) = match self.player {
            Sprite::Circle(x, y) => (x as f64, y as f64),
            _ => return,
        };

        for diamond in self.diamonds.iter_mut() {
            if let Sprite::Diamond(ref mut x, ref mut y) = diamond {
                let dx = player_x - *x;
                let dy = player_y - *y;
                let distance = (dx * dx + dy * dy).sqrt();

                if distance > ENEMY_SPEED {
                    *x = (*x + dx / distance * ENEMY_SPEED as f64).round();
                    *y = (*y + dy / distance * ENEMY_SPEED as f64).round();
                }
            }
        }
    }

    fn check_collisions(&mut self) {
        // check diamond-circle collision (= game over)
        if let Sprite::Circle(circle_x, circle_y) = self.player {
            for diamond in &self.diamonds {
                let diamond_vertices = diamond.get_vertices();
                if self.circle_collides_with_edges(
                    circle_x,
                    circle_y,
                    CIRCLE_RADIUS,
                    &diamond_vertices,
                ) {
                    println!("Collision with diamond!");
                    print!(
                        "Game over!\nScore: {}\nMultiplier: {}\n",
                        self.score, self.multiplier
                    );
                    // std::process::exit(0);
                    self.paused = true;
                }
            }
        };

        let (circle_x, circle_y, circle_radius) = match self.player {
            Sprite::Circle(x, y) => (x, y, CIRCLE_RADIUS),
            _ => panic!("Missing player sprite"),
        };

        let mut boom_triangles_indices = Vec::new();

        for (index, triangle) in self.triangles.iter().enumerate() {
            if let Sprite::Triangle(tx, ty, rot) = triangle {
                let vertices = self.get_triangle_vertices(*tx, *ty, *rot);

                if self.circle_collides_with_edges(circle_x, circle_y, circle_radius, &vertices) {
                    println!("Collision with triangle edge!");
                    boom_triangles_indices.push(index);
                }

                if self.circle_collides_with_triangle_corners(
                    circle_x,
                    circle_y,
                    circle_radius,
                    &vertices,
                ) {
                    println!("Collision with triangle corner!");
                    print!(
                        "Game over!\nScore: {}\nMultiplier: {}\n",
                        self.score, self.multiplier
                    );
                    // std::process::exit(0);
                    self.paused = true;
                }
            }
        }

        // Process the booms in reverse order to avoid index shifting issues
        for index in boom_triangles_indices.iter().rev() {
            let (tx, ty) = self.triangles[*index].get_coords();
            self.boom(tx, ty);
            self.triangles.remove(*index);
            self.pending_boom_locations.push((tx, ty));
        }

        // Now check multiplier collisions
        let (px, py) = self.player.get_coords();
        let consumed_multipliers = self.multipliers.extract_if(|mult| {
            let (mx, my) = mult.get_coords();
            let distance = ((px - mx).powi(2) + (py - my).powi(2)).sqrt();
            distance < (CIRCLE_RADIUS + SQUARE_RADIUS)
        });

        for _mult in consumed_multipliers {
            self.multiplier += 1;
        }
    }

    fn key_down(&mut self, key: String) {
        self.key_states.insert(key, true);
    }

    fn key_up(&mut self, key: String) {
        self.key_states.insert(key, false);
    }

    fn circle_collides_with_edges(
        &self,
        cx: f64,                 // Circle's center x-coordinate
        cy: f64,                 // Circle's center y-coordinate
        radius: f64,             // Circle's radius
        vertices: &[(f64, f64)], // Triangle vertices
    ) -> bool {
        vertices
            .iter()
            .zip(vertices.iter().cycle().skip(1))
            .any(|(&(x1, y1), &(x2, y2))| {
                // Calculate the vector components of the edge
                let dx = x2 - x1;
                let dy = y2 - y1;

                // Quadratic formula coefficients
                let a = dx * dx + dy * dy; // Coefficient of t^2
                let b = 2.0 * (dx * (x1 - cx) + dy * (y1 - cy)); // Coefficient of t
                let c = x1 * x1 + y1 * y1 + cx * cx + cy * cy
                    - 2.0 * (x1 * cx + y1 * cy)
                    - radius * radius; // Constant term

                // Discriminant of the quadratic equation
                let det = b * b - 4.0 * a * c;

                // Check for intersection: det > 0 indicates two solutions (intersections)
                // The intersection points must lie within the segment (0 <= t <= 1)
                det > 0.0 && -b / (2.0 * a) > 0.0 && -b / (2.0 * a) < 1.0
            })
    }

    fn circle_collides_with_triangle_corners(
        &self,
        cx: f64,
        cy: f64,
        radius: f64,
        vertices: &[(f64, f64)],
    ) -> bool {
        vertices.iter().any(|&(vx, vy)| {
            let dx = cx - vx;
            let dy = cy - vy;
            dx * dx + dy * dy < radius * radius
        })
    }

    fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }
}

#[tauri::command]
async fn event_loop(state: State<'_, AppState>, window: Window) -> Result<(), tauri::Error> {
    let mut last_enemy_spawn = std::time::Instant::now();
    let mut last_gate_spawn = std::time::Instant::now();

    loop {
        {
            let mut game = state.game.write().await;

            if last_enemy_spawn.elapsed() > Duration::from_millis(ENEMY_SPAWN_INTERVAL) {
                game.spawn_enemy();
                last_enemy_spawn = std::time::Instant::now();
            }

            if last_gate_spawn.elapsed() > Duration::from_millis(GATE_SPAWN_INTERVAL) {
                game.spawn_gate();
                last_gate_spawn = std::time::Instant::now();
            }
            game.tick();
            window.emit("update_sprites", &game.get_sprites())?;

            // and now check for explosions
            for (bx, by) in &game.pending_boom_locations {
                window.emit("explode", Sprite::Point(*bx, *by))?;
            }
            game.pending_boom_locations.clear();
        }

        sleep(Duration::from_millis(TICK_CYCLE_MS)).await;
    }
}

#[tauri::command]
async fn toggle_pause(state: State<'_, AppState>) -> Result<(), tauri::Error> {
    let mut game = state.game.write().await;
    game.toggle_pause();

    Ok(())
}

// Update the move_player command to key_down and key_up
#[tauri::command]
async fn key_down(state: State<'_, AppState>, key: String) -> Result<(), tauri::Error> {
    let mut game = state.game.write().await;
    game.key_down(key);

    Ok(())
}

#[tauri::command]
async fn key_up(state: State<'_, AppState>, key: String) -> Result<(), tauri::Error> {
    let mut game = state.game.write().await;
    game.key_up(key);

    Ok(())
}
#[tauri::command]
fn get_window_size() -> (f64, f64) {
    (WINDOW_WIDTH, WINDOW_HEIGHT)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            event_loop,
            key_up,
            key_down,
            toggle_pause,
            get_window_size
        ])
        .manage(AppState {
            game: RwLock::new(Game::new()),
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
