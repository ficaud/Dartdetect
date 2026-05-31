mod dart_core;
mod dart_calculator;
mod dart_game;
mod dart_interpretor;

use crate::dart_game::calculate_score_for_impact_point;
use crate::dart_calculator::Point;


// ----------------------------------------------------------------
// Solver performance notes
//
// Test case:
// Simulated impact point = { x: 250.0, y: 355.0 }
// Expected score = triple 20
//
// Baseline solver without gradient descent:
// Iterations: 125_606
// Processing time: 36.19 ms
//
// Solver with gradient descent:
// Iterations: 13
// Processing time: 297.125 us
// ----------------------------------------------------------------

fn main() {
    let impact = Point::new(250.0, 355.0);
    let payload = calculate_score_for_impact_point(impact)
        .expect("score should be computed");

    println!(
        "Reached impact point: Some(Point {{ x: {}, y: {} }})",
        payload.impact_x, payload.impact_y
    );
    println!("Score: {}", payload.score);
    println!("Processing time until score: {} us", payload.processing_time_us);
}
