use crate::constants::{
    CIRCLE_RADIUS, ENEMY_SPEED, MULTIPLIER_ATTRACT_MIN, MULTIPLIER_SPEED, PLAYER_SPEED,
    WINDOW_HEIGHT, WINDOW_WIDTH,
};
use crate::game::GameState;
use crate::sprites::Sprite;
use crate::traits::{Entity, Shape};

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
pub enum GameObject {
    Player(Sprite, GameObjectData),
    Gate(Sprite, GameObjectData),
    Enemy(Sprite, GameObjectData),
    Multiplier(Sprite, GameObjectData),
}

impl GameObject {
    pub fn get_sprite(&self) -> &Sprite {
        match self {
            GameObject::Player(sprite, _) => sprite,
            GameObject::Gate(sprite, _) => sprite,
            GameObject::Enemy(sprite, _) => sprite,
            GameObject::Multiplier(sprite, _) => sprite,
        }
    }

    pub fn new_player() -> Self {
        GameObject::Player(
            Sprite::Circle(WINDOW_WIDTH / 2.0, WINDOW_HEIGHT / 2.0),
            GameObjectData::new(),
        )
    }

    pub fn new_gate(coords: &(f64, f64), angle: f64, spin: f64) -> Self {
        GameObject::Gate(
            Sprite::Triangle(coords.0, coords.1, angle),
            GameObjectData {
                rotation_speed: Some(spin),
                spawn_time: Some(std::time::Instant::now()),
                ..GameObjectData::default()
            },
        )
    }

    pub fn new_enemy(coords: &(f64, f64)) -> Self {
        GameObject::Enemy(Sprite::Diamond(coords.0, coords.1), GameObjectData::new())
    }

    pub fn new_multiplier(coords: &(f64, f64), velocity: &(f64, f64)) -> Self {
        GameObject::Multiplier(
            Sprite::Square(coords.0, coords.1),
            GameObjectData {
                velocity: Some(*velocity),
                spawn_time: Some(std::time::Instant::now()),
                ..GameObjectData::default()
            },
        )
    }
}

impl Entity for GameObject {
    fn update(&mut self, game_state: GameState) {
        match self {
            GameObject::Gate(sprite, data) => {
                if let Sprite::Triangle(_, _, rot) = sprite {
                    *rot += data.rotation_speed.unwrap_or(1.0);
                }
            }

            GameObject::Player(sprite, _) => {
                if let Sprite::Circle(cx, cy) = sprite {
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
            }

            // Move enemy towards player
            GameObject::Enemy(sprite, _) => {
                if let Sprite::Diamond(ex, ey) = sprite {
                    let (px, py) = if let GameObject::Player(player_sprite, _) = game_state.player {
                        player_sprite.get_coords()
                    } else {
                        return;
                    };

                    let dx = px - *ex;
                    let dy = py - *ey;
                    let distance = (dx * dx + dy * dy).sqrt();
                    if distance > ENEMY_SPEED {
                        *ex += dx / distance * ENEMY_SPEED;
                        *ey += dy / distance * ENEMY_SPEED;
                    }
                }
            }

            // Move multiplier using velocity
            // or towards player
            GameObject::Multiplier(sprite, data) => {
                let (px, py) = if let GameObject::Player(player_sprite, _) = game_state.player {
                    player_sprite.get_coords()
                } else {
                    return;
                };

                if let Sprite::Square(mx, my) = sprite {
                    if let Some((vx, vy)) = data.velocity.as_mut() {
                        *mx += *vx;
                        *my += *vy;

                        // Apply decay to velocity
                        *vx *= 0.925;
                        *vy *= 0.925;
                    }

                    let dx = px - *mx;
                    let dy = py - *my;
                    let distance = (dx * dx + dy * dy).sqrt();
                    if distance < MULTIPLIER_ATTRACT_MIN {
                        *mx += dx / distance * MULTIPLIER_SPEED;
                        *my += dy / distance * MULTIPLIER_SPEED;
                    }
                }
            }
        }
    }
}
