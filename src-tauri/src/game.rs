use crate::collision::{check_corner_collision, check_edge_collision};
use crate::constants::{
    CIRCLE_RADIUS, ENEMY_BUFFER, EXPLOSION_RADIUS, GATE_BUFFER, SQUARE_RADIUS, WINDOW_HEIGHT,
    WINDOW_WIDTH,
};
use crate::sprites::Sprite;
use crate::traits::{Entity, Shape};

use rand::{thread_rng, Rng};
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct GameState {
    pub keys: HashSet<String>,
    pub player: Sprite,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            keys: HashSet::new(),
            player: Sprite::Circle(WINDOW_WIDTH / 2.0, WINDOW_HEIGHT / 2.0),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Game {
    game_state: GameState,
    sprites: Vec<Sprite>,
    pub score: u64,
    pub multiplier: u64,
    pub pending_boom_locations: Vec<(f64, f64)>,
    pub paused: bool,
    spawn_count: usize,
    pub game_over: bool,
}

impl Game {
    pub fn new() -> Self {
        Self {
            game_state: GameState::new(),
            sprites: Vec::new(),
            score: 0,
            multiplier: 1,
            pending_boom_locations: Vec::new(),
            paused: false,
            spawn_count: 1,
            game_over: false,
        }
    }

    pub fn reset_game(&mut self) {
        self.game_state = GameState::new();
        self.score = 0;
        self.multiplier = 1;
        self.spawn_count = 1;
        self.sprites.clear();
        self.game_over = false;
    }

    fn boom(&mut self, bx: f64, by: f64) {
        // TODO: send an explosion animation to the frontend
        let boomed_diamonds = self
            .sprites
            .extract_if(|sprite| {
                match sprite {
                    Sprite::Diamond(dx, dy) => {
                        // Keep the diamonds too far from the explosion
                        ((*dx - bx).powi(2) + (*dy - by).powi(2)).sqrt() < EXPLOSION_RADIUS
                    }
                    _ => false,
                }
            })
            .collect::<Vec<Sprite>>();

        for diamond in boomed_diamonds {
            let (dx, dy) = diamond.get_coords();
            // increment score and create a multiplier
            self.score += self.multiplier;
            self.sprites.push(Sprite::Square(dx, dy));
        }
    }

    pub fn get_sprites(&self) -> Vec<Sprite> {
        let mut sprites = vec![self.game_state.player.clone()];
        sprites.extend(self.sprites.clone());
        sprites
    }

    pub fn tick(&mut self) {
        self.game_state.player.update(self.game_state.clone());
        for sprite in &mut self.sprites {
            sprite.update(self.game_state.clone());
        }

        self.check_collisions();
    }

    pub fn spawn_enemy(&mut self) {
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

        for _ in 0..self.spawn_count {
            let x = rng.gen_range(x_min..x_max);
            let y = rng.gen_range(y_min..y_max);
            self.sprites.push(Sprite::Diamond(x, y));
        }

        self.spawn_count += 1;
    }

    pub fn spawn_gate(&mut self) {
        let mut rng = thread_rng();
        let gx = rng.gen_range(GATE_BUFFER..(WINDOW_WIDTH - GATE_BUFFER));
        let gy = rng.gen_range(GATE_BUFFER..(WINDOW_HEIGHT - GATE_BUFFER));
        let gr = rng.gen_range(0.0..360.0);
        self.sprites.push(Sprite::Triangle(gx, gy, gr))
    }

    fn check_collisions(&mut self) {
        let (cx, cy) = self.game_state.player.get_coords();

        // check enemy-player collision (=> game over, you lose, good day sir!)
        for sprite in &self.sprites {
            match sprite {
                Sprite::Diamond(_, _) => {
                    if check_edge_collision(sprite, &self.game_state) {
                        println!("Collision with enemy!");
                        print!(
                            "Game over!\nScore: {}\nMultiplier: {}\n",
                            self.score, self.multiplier
                        );
                        self.game_over = true;
                    }
                }
                _ => {}
            }
        }

        let triangles_to_boom = self
            .sprites
            .extract_if(|sprite| match sprite {
                Sprite::Triangle(_, _, _) => check_edge_collision(sprite, &self.game_state),
                _ => false,
            })
            .collect::<Vec<Sprite>>();

        for sprite in &self.sprites {
            match sprite {
                Sprite::Triangle(_, _, _) => {
                    if check_corner_collision(sprite, &self.game_state) {
                        println!("Collision with triangle corner!");
                        print!(
                            "Game over!\nScore: {}\nMultiplier: {}\n",
                            self.score, self.multiplier
                        );
                        self.game_over = true;
                    }
                }
                _ => {}
            }
        }

        for triangle in triangles_to_boom {
            let (triangle_x, triangle_y) = triangle.get_coords();
            self.boom(triangle_x, triangle_y);
            self.pending_boom_locations.push((triangle_x, triangle_y));
        }

        // Now check multiplier collisions
        let squares_to_consume = self
            .sprites
            .extract_if(|sprite| match sprite {
                Sprite::Square(_, _) => {
                    let (mx, my) = sprite.get_coords();
                    let dx = cx - mx;
                    let dy = cy - my;
                    let distance = (dx * dx + dy * dy).sqrt();
                    distance < CIRCLE_RADIUS + SQUARE_RADIUS
                }
                _ => false,
            })
            .collect::<Vec<Sprite>>();

        for _square in squares_to_consume {
            self.multiplier += 1;
        }
    }

    pub fn key_down(&mut self, key: String) {
        self.game_state.keys.insert(key);
    }

    pub fn key_up(&mut self, key: String) {
        self.game_state.keys.remove(&key);
    }
}
