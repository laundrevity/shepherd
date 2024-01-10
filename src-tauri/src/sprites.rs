use crate::constants::{DIAMOND_RADIUS, SQUARE_RADIUS, TRIANGLE_RADIUS};
use crate::traits::Shape;

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub enum Sprite {
    Triangle(f64, f64, f64), // x coordinate, y coordinate, rotation
    Circle(f64, f64),
    Diamond(f64, f64),
    Square(f64, f64),
    Point(f64, f64),
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
