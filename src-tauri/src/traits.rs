use crate::game::GameState;

pub trait Entity {
    fn update(&mut self, game_state: GameState);
}

pub trait Shape {
    fn get_coords(&self) -> (f64, f64);
    fn get_vertices(&self) -> Vec<(f64, f64)>;
}
