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

const PLAYER_SPEED: f64 = 3.0;
const ENEMY_SPEED: f64 = 2.0;
const MULTIPLIER_SPEED: f64 = 2.9;

const TICK_CYCLE_MS: u64 = 8;
const ENEMY_SPAWN_INTERVAL: u64 = 5000;
const GATE_SPAWN_INTERVAL: u64 = 10000;

const CIRCLE_RADIUS: f64 = 15.0;
const DIAMOND_RADIUS: f64 = 25.0;
const TRIANGLE_RADIUS: f64 = 75.0;
const SQUARE_RADIUS: f64 = 5.0;

const EXPLOSION_RADIUS: f64 = 150.0;
const GATE_BUFFER: f64 = 25.0;
const ENEMY_BUFFER: f64 = 150.0;
const MULTIPLIER_ATTRACT_MIN: f64 = 75.0;

#[derive(Serialize)]
struct GameConstants {
    window_width: f64,
    window_height: f64,
    circle_radius: f64,
    diamond_radius: f64,
    triangle_radius: f64,
    square_radius: f64,
    explosion_radius: f64,
}

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
struct GameData {
    score: u64,
    multiplier: u64,
    key_states: HashMap<String, bool>,
    pending_boom_locations: Vec<(f64, f64)>,
    paused: bool,
    spawn_count: usize,
    over: bool,
}

impl GameData {
    fn new() -> Self {
        Self {
            score: 0,
            multiplier: 1,
            key_states: HashMap::new(),
            pending_boom_locations: Vec::new(),
            paused: false,
            spawn_count: 1,
            over: false,
        }
    }
}

#[derive(Clone, Debug)]
struct Game {
    player: Sprite,
    triangles: Vec<Sprite>,
    diamonds: Vec<Sprite>,
    multipliers: Vec<Sprite>,
    game_data: GameData,
}

struct AppState {
    game: RwLock<Game>,
}

impl Game {
    fn new() -> Self {
        Self {
            player: Sprite::Circle(200.0, 200.0),
            triangles: Vec::new(),
            diamonds: Vec::new(),
            multipliers: Vec::new(),
            game_data: GameData::new(),
        }
    }

    fn reset_game(&mut self) {
        self.player = Sprite::Circle(WINDOW_WIDTH/2.0, WINDOW_HEIGHT/2.0);
        self.triangles.clear();
        self.diamonds.clear();
        self.multipliers.clear();
        self.game_data = GameData::new();
    }

    fn boom(&mut self, bx: f64, by: f64) {
        // TODO: send an explosion animation to the frontend
        let boomed_diamonds = self
            .diamonds
            .extract_if(|diamond| {
                let (dx, dy) = diamond.get_coords();
                // Keep the diamonds too far from the explosion
                ((dx - bx).powi(2) + (dy - by).powi(2)).sqrt() < EXPLOSION_RADIUS
            })
            .collect::<Vec<Sprite>>();

        for diamond in boomed_diamonds {
            let (dx, dy) = diamond.get_coords();
            // increment score and create a multiplier
            self.game_data.score += self.game_data.multiplier;
            self.multipliers.push(Sprite::Square(dx, dy));
        }
    }

    fn get_sprites(&self) -> Vec<Sprite> {
        let mut sprites = vec![self.player.clone()];
        sprites.extend(self.triangles.clone());
        sprites.extend(self.diamonds.clone());
        sprites.extend(self.multipliers.clone());
        sprites
    }

    fn tick(&mut self) {
        self.rotate_triangles();
        self.move_player();
        self.move_enemies();
        self.move_multipliers();

        self.check_collisions();
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

            if *self.game_data.key_states.get("w").unwrap_or(&false) {
                dy -= PLAYER_SPEED as f64;
            }
            if *self.game_data.key_states.get("a").unwrap_or(&false) {
                dx -= PLAYER_SPEED as f64;
            }
            if *self.game_data.key_states.get("s").unwrap_or(&false) {
                dy += PLAYER_SPEED as f64;
            }
            if *self.game_data.key_states.get("d").unwrap_or(&false) {
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

        let (x_min, x_max, y_min, y_max) = match rng.gen_range(0..4) {
            0 => (0.0, ENEMY_BUFFER, 0.0, ENEMY_BUFFER),
            1 => (WINDOW_WIDTH - ENEMY_BUFFER, WINDOW_WIDTH, 0.0, ENEMY_BUFFER),
            2 => (
                WINDOW_WIDTH - ENEMY_BUFFER,
                WINDOW_WIDTH,
                WINDOW_HEIGHT - ENEMY_BUFFER,
                WINDOW_HEIGHT,
            ),
            _ => (
                0.0,
                ENEMY_BUFFER,
                WINDOW_HEIGHT - ENEMY_BUFFER,
                WINDOW_HEIGHT,
            ),
        };

        for _ in 0..self.game_data.spawn_count {
            let x = rng.gen_range(x_min..x_max);
            let y = rng.gen_range(y_min..y_max);
            self.diamonds.push(Sprite::Diamond(x, y));
        }

        self.game_data.spawn_count += 1;
    }

    fn spawn_gate(&mut self) {
        let mut rng = thread_rng();
        let gx = rng.gen_range(GATE_BUFFER..(WINDOW_WIDTH - GATE_BUFFER));
        let gy = rng.gen_range(GATE_BUFFER..(WINDOW_HEIGHT - GATE_BUFFER));
        let gr = rng.gen_range(0.0..360.0);
        self.triangles.push(Sprite::Triangle(gx, gy, gr))
    }

    fn move_enemies(&mut self) {
        let (player_x, player_y) = self.player.get_coords();

        for diamond in self.diamonds.iter_mut() {
            if let Sprite::Diamond(ref mut x, ref mut y) = diamond {
                let dx = player_x - *x;
                let dy = player_y - *y;
                let distance = (dx * dx + dy * dy).sqrt();

                if distance > ENEMY_SPEED {
                    *x = *x + dx / distance * ENEMY_SPEED;
                    *y = *y + dy / distance * ENEMY_SPEED;
                }
            }
        }
    }

    fn move_multipliers(&mut self) {
        let (player_x, player_y) = self.player.get_coords();

        for mult in self.multipliers.iter_mut() {
            if let Sprite::Square(ref mut x, ref mut y) = mult {
                let dx = player_x - *x;
                let dy = player_y - *y;
                let distance = (dx * dx + dy * dy).sqrt();

                if distance < MULTIPLIER_ATTRACT_MIN {
                    *x = *x + dx / distance * MULTIPLIER_SPEED;
                    *y = *y + dy / distance * MULTIPLIER_SPEED;
                }
            }
        }
    }

    fn check_collisions(&mut self) {
        // check diamond-circle collision (= game over)
        if let Sprite::Circle(circle_x, circle_y) = self.player {
            for diamond in &self.diamonds {
                let diamond_vertices = diamond.get_vertices();
                if Game::circle_collides_with_edges(
                    circle_x,
                    circle_y,
                    CIRCLE_RADIUS,
                    &diamond_vertices,
                ) {
                    println!("Collision with diamond!");
                    print!(
                        "Game over!\nScore: {}\nMultiplier: {}\n",
                        self.game_data.score, self.game_data.multiplier
                    );
                    self.game_data.over = true;
                }
            }
        };

        let (circle_x, circle_y) = self.player.get_coords();

        let triangles_to_boom = self
            .triangles
            .extract_if(|triangle| {
                let vertices = triangle.get_vertices();

                Game::circle_collides_with_edges(circle_x, circle_y, CIRCLE_RADIUS, &vertices)
            })
            .collect::<Vec<Sprite>>();

        for triangle in &self.triangles {
            let vertices = triangle.get_vertices();
            if self.circle_collides_with_triangle_corners(
                circle_x,
                circle_y,
                CIRCLE_RADIUS,
                &vertices,
            ) {
                println!("Collision with triangle corner!");
                print!(
                    "Game over!\nScore: {}\nMultiplier: {}\n",
                    self.game_data.score, self.game_data.multiplier
                );
                self.game_data.over = true;
            }
        }

        for triangle in triangles_to_boom {
            let (triangle_x, triangle_y) = triangle.get_coords();
            self.boom(triangle_x, triangle_y);
            self.game_data.pending_boom_locations.push((triangle_x, triangle_y));
        }

        // Now check multiplier collisions
        let (px, py) = self.player.get_coords();
        let consumed_multipliers = self.multipliers.extract_if(|mult| {
            let (mx, my) = mult.get_coords();
            let distance = ((px - mx).powi(2) + (py - my).powi(2)).sqrt();
            distance < (CIRCLE_RADIUS + SQUARE_RADIUS)
        });

        for _mult in consumed_multipliers {
            self.game_data.multiplier += 1;
        }
    }

    fn key_down(&mut self, key: String) {
        self.game_data.key_states.insert(key, true);
    }

    fn key_up(&mut self, key: String) {
        self.game_data.key_states.insert(key, false);
    }

    fn circle_collides_with_edges(
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
}

#[tauri::command]
async fn event_loop(state: State<'_, AppState>, window: Window) -> Result<(), tauri::Error> {
    let mut last_enemy_spawn = std::time::Instant::now();
    let mut last_gate_spawn = std::time::Instant::now();

    loop {
        {
            let mut game = state.game.write().await;

            if !game.game_data.paused && !game.game_data.over {
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
                for (bx, by) in &game.game_data.pending_boom_locations {
                    window.emit("explode", Sprite::Point(*bx, *by))?;
                }
                game.game_data.pending_boom_locations.clear();

                // Emit score and multiplier updates to the frontend
                window.emit("update_score_multiplier", (&game.game_data.score, &game.game_data.multiplier))?;
            }
        }

        sleep(Duration::from_millis(TICK_CYCLE_MS)).await;
    }
}

#[tauri::command]
async fn handle_spacebar(state: State<'_, AppState>) -> Result<(), tauri::Error> {
    let mut game = state.game.write().await;

    if game.game_data.over {
        game.reset_game();
    } else {
        game.game_data.paused = !game.game_data.paused;
    }

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
fn get_game_constants() -> GameConstants {
    GameConstants {
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        circle_radius: CIRCLE_RADIUS,
        diamond_radius: DIAMOND_RADIUS,
        triangle_radius: TRIANGLE_RADIUS,
        square_radius: SQUARE_RADIUS,
        explosion_radius: EXPLOSION_RADIUS,
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            event_loop,
            key_up,
            key_down,
            handle_spacebar,
            get_game_constants
        ])
        .manage(AppState {
            game: RwLock::new(Game::new()),
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
