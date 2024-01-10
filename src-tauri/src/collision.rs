use crate::constants::CIRCLE_RADIUS;
use crate::game::GameState;
use crate::traits::Shape;

pub fn check_edge_collision<S: Shape>(shape: &S, game_state: &GameState) -> bool {
    let (cx, cy) = game_state.player.get_sprite().get_coords();
    let vertices = shape.get_vertices();

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
                - CIRCLE_RADIUS * CIRCLE_RADIUS; // Constant term

            // Discriminant of the quadratic equation
            let det = b * b - 4.0 * a * c;

            // Check for intersection: det > 0 indicates two solutions (intersections)
            // The intersection points must lie within the segment (0 <= t <= 1)
            det > 0.0 && -b / (2.0 * a) > 0.0 && -b / (2.0 * a) < 1.0
        })
}

pub fn check_corner_collision<S: Shape>(shape: &S, game_state: &GameState) -> bool {
    let (cx, cy) = game_state.player.get_sprite().get_coords();
    let vertices = shape.get_vertices();

    vertices.iter().any(|&(vx, vy)| {
        let dx = cx - vx;
        let dy = cy - vy;
        dx * dx + dy * dy < CIRCLE_RADIUS * CIRCLE_RADIUS
    })
}
