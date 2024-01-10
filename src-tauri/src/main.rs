// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(extract_if)]

mod collision;
mod constants;
mod game;
mod game_objects;
mod sprites;
mod traits;

use crate::constants::{GameConstants, ENEMY_SPAWN_INTERVAL, GATE_SPAWN_INTERVAL, TICK_CYCLE_MS};
use crate::game::Game;
use crate::sprites::Sprite;

use tauri::{async_runtime::RwLock, State, Window};
use tokio::time::{sleep, Duration};

struct AppState {
    game: RwLock<Game>,
}

#[tauri::command]
async fn event_loop(state: State<'_, AppState>, window: Window) -> Result<(), tauri::Error> {
    let mut last_enemy_spawn = std::time::Instant::now();
    let mut last_gate_spawn = std::time::Instant::now();

    loop {
        {
            let mut game = state.game.write().await;

            if !game.paused && !game.game_over {
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

                // Emit score and multiplier updates to the frontend
                window.emit("update_score_multiplier", (&game.score, &game.multiplier))?;
            }
        }

        sleep(Duration::from_millis(TICK_CYCLE_MS)).await;
    }
}

#[tauri::command]
async fn handle_spacebar(state: State<'_, AppState>) -> Result<(), tauri::Error> {
    let mut game = state.game.write().await;

    if game.game_over {
        game.reset_game();
    } else {
        game.paused = !game.paused;
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
    GameConstants::new()
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
