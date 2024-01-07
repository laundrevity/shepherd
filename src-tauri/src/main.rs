// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rand::{thread_rng, Rng};
use serde::Serialize;
use std::collections::HashMap;
use tauri::{async_runtime::RwLock, State, Window};
use tokio::time::{sleep, Duration};

const WINDOW_WIDTH: usize = 1200;
const WINDOW_HEIGHT: usize = 800;

const PLAYER_SPEED: usize = 5;
const ENEMY_SPEED: usize = 2;

const TICK_CYCLE_MS: u64 = 16;
const SPAWN_INTERVAL: u64 = 5000;

#[derive(Clone, Debug, Serialize)]
enum Sprite {
    Triangle(usize, usize, usize), // x coordinate, y coordinate, rotation
    Circle(usize, usize),
    Diamond(usize, usize),
}

#[derive(Clone, Debug)]
struct Game {
    player: Sprite,
    triangles: Vec<Sprite>,
    diamonds: Vec<Sprite>,
    key_states: HashMap<String, bool>,
}

struct AppState {
    game: RwLock<Game>,
}

impl Game {
    fn new() -> Self {
        Self {
            player: Sprite::Circle(200, 200),
            triangles: vec![Sprite::Triangle(100, 100, 0)],
            diamonds: Vec::new(),
            key_states: HashMap::new(),
        }
    }

    fn get_sprites(&self) -> Vec<Sprite> {
        let mut sprites = vec![self.player.clone()];
        sprites.extend(self.triangles.clone());
        sprites.extend(self.diamonds.clone());
        sprites
    }

    fn tick(&mut self) {
        // Rotate the triangle
        for triangle in self.triangles.iter_mut() {
            if let Sprite::Triangle(_, _, ref mut rot) = triangle {
                *rot = (*rot + 1) % 360;
            }
        }

        // Update circle position based on key states
        if let Sprite::Circle(ref mut x, ref mut y) = self.player {
            if *self.key_states.get("w").unwrap_or(&false) {
                *y = y.saturating_sub(PLAYER_SPEED).max(20);
            }
            if *self.key_states.get("a").unwrap_or(&false) {
                *x = x.saturating_sub(PLAYER_SPEED).max(20);
            }
            if *self.key_states.get("s").unwrap_or(&false) {
                *y = (*y + PLAYER_SPEED).min(WINDOW_HEIGHT - 20);
            }
            if *self.key_states.get("d").unwrap_or(&false) {
                *x = (*x + PLAYER_SPEED).min(WINDOW_WIDTH - 20);
            }
        }

        self.move_enemies();
    }

    fn spawn_enemy(&mut self) {
        let mut rng = thread_rng();
        let (x, y) = match rng.gen_range(0..4) {
            0 => (0, 0),
            1 => (WINDOW_WIDTH, 0),
            2 => (0, WINDOW_HEIGHT),
            _ => (WINDOW_WIDTH, WINDOW_HEIGHT),
        };

        self.diamonds.push(Sprite::Diamond(x, y));
    }

    fn move_enemies(&mut self) {
        let (player_x, player_y) = if let Sprite::Circle(x, y) = self.player {
            (x, y)
        } else {
            return;
        };

        self.diamonds
            .iter_mut()
            .filter_map(|sprite| {
                if let Sprite::Diamond(ref mut x, ref mut y) = sprite {
                    Some((x, y))
                } else {
                    None
                }
            })
            .for_each(|(x, y)| {
                if *x < player_x {
                    *x += ENEMY_SPEED.min(player_x - *x);
                } else if *x > player_x {
                    *x -= ENEMY_SPEED.min(*x - player_x);
                }

                if *y < player_y {
                    *y += ENEMY_SPEED.min(player_y - *y);
                } else if *y > player_y {
                    *y -= ENEMY_SPEED.min(*y - player_y);
                }
            });
    }

    // Add methods to update key states
    fn key_down(&mut self, key: String) {
        self.key_states.insert(key, true);
    }

    fn key_up(&mut self, key: String) {
        self.key_states.insert(key, false);
    }
}

#[tauri::command]
async fn event_loop(state: State<'_, AppState>, window: Window) -> Result<(), tauri::Error> {
    let mut last_spawn = std::time::Instant::now();

    loop {
        {
            let mut game = state.game.write().await;

            if last_spawn.elapsed() > Duration::from_millis(SPAWN_INTERVAL) {
                game.spawn_enemy();
                last_spawn = std::time::Instant::now();
            }

            game.tick();
            window.emit("update_sprites", &game.get_sprites())?;
        }

        sleep(Duration::from_millis(TICK_CYCLE_MS)).await;
    }
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
fn get_window_size() -> (usize, usize) {
    (WINDOW_WIDTH, WINDOW_HEIGHT)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            event_loop,
            key_up,
            key_down,
            get_window_size
        ])
        .manage(AppState {
            game: RwLock::new(Game::new()),
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
