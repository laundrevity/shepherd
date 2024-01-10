use crate::collision::{check_corner_collision, check_edge_collision};
use crate::constants::{
    CIRCLE_RADIUS, ENEMY_BUFFER_FRAC, EXPLOSION_RADIUS, GATE_BUFFER, MULTIPLIER_LIFETIME_MS,
    SQUARE_RADIUS, WINDOW_HEIGHT, WINDOW_WIDTH,
};
use crate::game_objects::GameObject;
use crate::sprites::Sprite;
use crate::traits::{Entity, Shape};

use rand::{thread_rng, Rng};
use std::collections::HashSet;

impl GameState {
    pub fn new() -> Self {
        Self {
            keys: HashSet::new(),
            player: GameObject::new_player(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GameState {
    pub keys: HashSet<String>,
    pub player: GameObject,
}

#[derive(Clone, Debug)]
pub struct Game {
    game_state: GameState,
    game_objects: Vec<GameObject>,
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
            game_objects: Vec::new(),
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
        self.game_objects.clear();
        self.game_over = false;
    }

    fn boom(&mut self, bx: f64, by: f64) {
        let boom_strength = 500.0; // Adjust this constant based on desired effect
        let epsilon = 1.0;

        let boomed_diamonds = self
            .game_objects
            .extract_if(|game_object| match game_object {
                GameObject::Enemy(sprite, _) => {
                    if let Sprite::Diamond(ex, ey) = sprite {
                        ((*ex - bx).powi(2) + (*ey - by).powi(2)).sqrt() < EXPLOSION_RADIUS
                    } else {
                        false
                    }
                }
                _ => false,
            })
            .collect::<Vec<GameObject>>();

        for diamond_object in boomed_diamonds {
            let (dx, dy) = diamond_object.get_sprite().get_coords();
            let distance = ((dx - bx).powi(2) + (dy - by).powi(2)).sqrt();

            let velocity_magnitude = boom_strength / (distance + epsilon);
            let direction_x = (dx - bx) / distance;
            let direction_y = (dy - by) / distance;

            let velocity_x = direction_x * velocity_magnitude;
            let velocity_y = direction_y * velocity_magnitude;

            // increment score and create a multiplier
            self.score += self.multiplier;
            self.game_objects.push(GameObject::new_multiplier(
                &(dx, dy),
                &(velocity_x, velocity_y),
            ));
        }
    }

    pub fn get_sprites(&self) -> Vec<Sprite> {
        let mut sprites = vec![self.game_state.player.get_sprite().clone()];
        for game_object in &self.game_objects {
            sprites.push(game_object.get_sprite().clone())
        }
        sprites
    }

    pub fn tick(&mut self) {
        self.game_state.player.update(self.game_state.clone());
        for sprite_data in &mut self.game_objects {
            sprite_data.update(self.game_state.clone());
        }

        self.check_collisions();
        self.cull();
    }

    pub fn spawn_enemy(&mut self) {
        let mut rng = thread_rng();
        let horizontal_buffer = ENEMY_BUFFER_FRAC * WINDOW_WIDTH;
        let vertical_buffer = ENEMY_BUFFER_FRAC * WINDOW_HEIGHT;
        let (x_min, x_max, y_min, y_max) = match rng.gen_range(0..4) {
            0 => (0.0, horizontal_buffer, 0.0, vertical_buffer),
            1 => (
                WINDOW_WIDTH - horizontal_buffer,
                WINDOW_WIDTH,
                0.0,
                vertical_buffer,
            ),
            2 => (
                WINDOW_WIDTH - horizontal_buffer,
                WINDOW_WIDTH,
                WINDOW_HEIGHT - vertical_buffer,
                WINDOW_HEIGHT,
            ),
            _ => (
                0.0,
                horizontal_buffer,
                WINDOW_HEIGHT - vertical_buffer,
                WINDOW_HEIGHT,
            ),
        };

        for _ in 0..self.spawn_count {
            let x = rng.gen_range(x_min..x_max);
            let y = rng.gen_range(y_min..y_max);
            self.game_objects.push(GameObject::new_enemy(&(x, y)));
        }

        self.spawn_count += 1;
    }

    pub fn spawn_gate(&mut self) {
        let mut rng = thread_rng();
        let gx = rng.gen_range(GATE_BUFFER..(WINDOW_WIDTH - GATE_BUFFER));
        let gy = rng.gen_range(GATE_BUFFER..(WINDOW_HEIGHT - GATE_BUFFER));
        let gr = rng.gen_range(0.0..360.0);
        let gate_spin = rng.gen_range(-1.0..1.0);
        self.game_objects
            .push(GameObject::new_gate(&(gx, gy), gr, gate_spin));
    }

    fn check_collisions(&mut self) {
        let (cx, cy) = self.game_state.player.get_sprite().get_coords();

        let triangles_to_boom = self
            .game_objects
            .extract_if(|game_object| match game_object.get_sprite() {
                Sprite::Triangle(_, _, _) => {
                    check_edge_collision(game_object.get_sprite(), &self.game_state)
                }
                _ => false,
            })
            .collect::<Vec<GameObject>>();

        if triangles_to_boom.is_empty() {
            for game_object in &self.game_objects {
                match game_object {
                    GameObject::Gate(sprite, data) => {
                        if data
                            .spawn_time
                            .unwrap_or(std::time::Instant::now())
                            .elapsed()
                            > std::time::Duration::from_millis(5000)
                        {
                            if check_corner_collision(sprite, &self.game_state) {
                                println!("Collision with triangle corner!");
                                print!(
                                    "Game over!\nScore: {}\nMultiplier: {}\n",
                                    self.score, self.multiplier
                                );
                                self.game_over = true;
                            }
                        }
                    }
                    _ => {}
                }
            }

            // check enemy-player collision (=> game over, you lose, good day sir!)
            for game_object in &self.game_objects {
                match game_object {
                    GameObject::Enemy(sprite, _) => {
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
        } else {
            for triangle in triangles_to_boom {
                let (triangle_x, triangle_y) = triangle.get_sprite().get_coords();
                self.boom(triangle_x, triangle_y);
                self.pending_boom_locations.push((triangle_x, triangle_y));
            }
        }

        // Now check multiplier collisions
        let squares_to_consume = self
            .game_objects
            .extract_if(|game_object| match game_object {
                GameObject::Multiplier(sprite, _) => {
                    let (mx, my) = sprite.get_coords();
                    let dx = cx - mx;
                    let dy = cy - my;
                    let distance = (dx * dx + dy * dy).sqrt();
                    distance < CIRCLE_RADIUS + SQUARE_RADIUS
                }
                _ => false,
            })
            .collect::<Vec<GameObject>>();

        for _square in squares_to_consume {
            self.multiplier += 1;
        }
    }

    fn cull(&mut self) {
        self.game_objects.retain(|game_object| match game_object {
            GameObject::Multiplier(_, data) => {
                data.spawn_time
                    .unwrap_or(std::time::Instant::now())
                    .elapsed()
                    < std::time::Duration::from_millis(MULTIPLIER_LIFETIME_MS)
            }
            _ => true,
        });
    }

    pub fn key_down(&mut self, key: String) {
        self.game_state.keys.insert(key);
    }

    pub fn key_up(&mut self, key: String) {
        self.game_state.keys.remove(&key);
    }
}
