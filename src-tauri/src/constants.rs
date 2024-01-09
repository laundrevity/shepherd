use serde::Serialize;

pub const WINDOW_WIDTH: f64 = 1200.0;
pub const WINDOW_HEIGHT: f64 = 800.0;

pub const PLAYER_SPEED: f64 = 3.0;
pub const ENEMY_SPEED: f64 = 2.0;
pub const MULTIPLIER_SPEED: f64 = 2.9;

pub const TICK_CYCLE_MS: u64 = 8;
pub const ENEMY_SPAWN_INTERVAL: u64 = 5000;
pub const GATE_SPAWN_INTERVAL: u64 = 10000;

pub const CIRCLE_RADIUS: f64 = 15.0;
pub const DIAMOND_RADIUS: f64 = 25.0;
pub const TRIANGLE_RADIUS: f64 = 75.0;
pub const SQUARE_RADIUS: f64 = 5.0;

pub const EXPLOSION_RADIUS: f64 = 150.0;
pub const GATE_BUFFER: f64 = 25.0;
pub const ENEMY_BUFFER: f64 = 150.0;
pub const MULTIPLIER_ATTRACT_MIN: f64 = 75.0;

#[derive(Serialize)]
pub struct GameConstants {
    window_width: f64,
    window_height: f64,
    circle_radius: f64,
    diamond_radius: f64,
    triangle_radius: f64,
    square_radius: f64,
    explosion_radius: f64,
}

impl GameConstants {
    pub fn new() -> Self {
        Self {
            window_width: WINDOW_WIDTH,
            window_height: WINDOW_HEIGHT,
            circle_radius: CIRCLE_RADIUS,
            diamond_radius: DIAMOND_RADIUS,
            triangle_radius: TRIANGLE_RADIUS,
            square_radius: SQUARE_RADIUS,
            explosion_radius: EXPLOSION_RADIUS,
        }
    }
}
