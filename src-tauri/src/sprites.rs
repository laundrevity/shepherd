use crate::constants::{
    CIRCLE_RADIUS, DIAMOND_RADIUS, ENEMY_SPEED, MULTIPLIER_ATTRACT_MIN, MULTIPLIER_SPEED,
    PLAYER_SPEED, SQUARE_RADIUS, TRIANGLE_RADIUS, WINDOW_HEIGHT, WINDOW_WIDTH,
};
use crate::game::GameState;
use crate::traits::{Entity, Shape};

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub enum Sprite {
    Triangle(f64, f64, f64), // x coordinate, y coordinate, rotation
    Circle(f64, f64),
    Diamond(f64, f64),
    Square(f64, f64),
    Point(f64, f64),
}

#[derive(Clone, Debug, Default)]
pub struct GameObjectData {
    pub rotation_speed: Option<f64>,
    pub velocity: Option<(f64, f64)>,
    pub spawn_time: Option<std::time::Instant>,
}

impl GameObjectData {
    fn new() -> Self {
        Self {
            rotation_speed: None,
            velocity: None,
            spawn_time: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GameObject {
    pub sprite: Sprite,
    pub data: GameObjectData,
}

impl GameObject {
    pub fn new(sprite: Sprite) -> Self {
        Self {
            sprite,
            data: GameObjectData::new(),
        }
    }
}

impl Entity for GameObject {
    fn update(&mut self, game_state: GameState) {
        match &mut self.sprite {
            // Rotate gate
            Sprite::Triangle(_, _, rotation) => {
                *rotation += self.data.rotation_speed.unwrap_or(1.0);
            }

            // Move player according to WASD states
            Sprite::Circle(cx, cy) => {
                let mut dx = 0.0;
                let mut dy = 0.0;

                if game_state.keys.contains("w") {
                    dy -= PLAYER_SPEED;
                }

                if game_state.keys.contains("a") {
                    dx -= PLAYER_SPEED;
                }

                if game_state.keys.contains("s") {
                    dy += PLAYER_SPEED;
                }

                if game_state.keys.contains("d") {
                    dx += PLAYER_SPEED;
                }

                // Scale speed for diagonal movement
                let speed_scale = if dx != 0.0 && dy != 0.0 {
                    (2f64).sqrt() / 2.0
                } else {
                    1.0
                };

                *cx = (*cx + dx * speed_scale)
                    .max(CIRCLE_RADIUS)
                    .min(WINDOW_WIDTH - CIRCLE_RADIUS);

                *cy = (*cy + dy * speed_scale)
                    .max(CIRCLE_RADIUS)
                    .min(WINDOW_HEIGHT - CIRCLE_RADIUS);
            }

            // Move enemy towards player
            Sprite::Diamond(ex, ey) => {
                let (px, py) = game_state.player.sprite.get_coords();

                let dx = px - *ex;
                let dy = py - *ey;
                let distance = (dx * dx + dy * dy).sqrt();
                if distance > ENEMY_SPEED {
                    *ex += dx / distance * ENEMY_SPEED;
                    *ey += dy / distance * ENEMY_SPEED;
                }
            }

            // Move multiplier using velocity
            // or towards player
            Sprite::Square(mx, my) => {
                if let Some((vx, vy)) = self.data.velocity.as_mut() {
                    *mx += *vx;
                    *my += *vy;

                    // Apply decay to velocity
                    *vx *= 0.925;
                    *vy *= 0.925;
                }

                let (px, py) = game_state.player.sprite.get_coords();

                let dx = px - *mx;
                let dy = py - *my;
                let distance = (dx * dx + dy * dy).sqrt();
                if distance < MULTIPLIER_ATTRACT_MIN {
                    *mx += dx / distance * MULTIPLIER_SPEED;
                    *my += dy / distance * MULTIPLIER_SPEED;
                }
            }
            _ => {}
        }
    }
}

impl Shape for Sprite {
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
