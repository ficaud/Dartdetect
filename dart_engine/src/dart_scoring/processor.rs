use crate::dart_calculator::{ImpactLocator, ImpactSimulator, Point, SensorPos};
use crate::dart_interpretor::score::Score;

// We assume the board as a size of
//
// 500mm x 500mm, with the origin (0,0)
// at the bottom-left corner and (500,500) at the top-right corner.
//
// From that assumption we have 4 sensors at the corners of the board.
const BOARD_SIZE: f64 = 500.0;

// Default sensor positions at the corners of the board.
fn default_sensors() -> [SensorPos; 4] {
    [
        SensorPos::C1(Point::new(0.0, BOARD_SIZE)),
        SensorPos::C2(Point::new(BOARD_SIZE, BOARD_SIZE)),
        SensorPos::C3(Point::new(BOARD_SIZE, 0.0)),
        SensorPos::C4(Point::new(0.0, 0.0)),
    ]
}

/// Calculate the score for a given impact point on the dartboard.
///
/// # Arguments
/// - impact_point: A `Point` struct representing the x and y coordinates of the dart's impact on the board.
///
/// This function simulates the sensor timings for the given impact point,
/// then uses the `ImpactLocator` to find the impact point based on those timings,
/// and finally calculates the score using the `Score` struct.
pub fn calculate_score_for_impact_point(impact_point: Point) -> Result<u32, String> {
    // Simulate the sensor timings for the given impact point using the `ImpactSimulator`.
    let sensors = default_sensors();
    let baseline_time_us = 1_000.0;
    let impact_simulator = ImpactSimulator::from_point(&impact_point, &sensors, baseline_time_us);
    let locator = ImpactLocator::new(
        sensors,
        [
            impact_simulator.t1,
            impact_simulator.t2,
            impact_simulator.t3,
            impact_simulator.t4,
        ],
    );

    println!(
        "Simulated imapcts delays (us): t1={}, t2={}, t3={}, t4={}",
        impact_simulator.t1, impact_simulator.t2, impact_simulator.t3, impact_simulator.t4
    );

    // Use the `ImpactLocator` to find the impact point based on the simulated timings.
    let reached_impact = locator
        .locate()
        .ok_or_else(|| String::from("solver did not find an impact point"))?;

    // Calculate the score using the `Score` struct based on the reached impact point.
    let mut score = Score::new(reached_impact);
    Ok(score.get_score())
}

/// Calculate the score for given sensor timings.
///
/// # Arguments
/// - t1: Timing from sensor 1 in mmicroseconds
/// - t2: Timing from sensor 2 in microseconds
/// - t3: Timing from sensor 3 in microseconds
/// - t4: Timing from sensor 4 in microseconds
///
/// This function uses the `ImpactLocator` to find the impact point based on the provided sensor timings,
pub fn calculate_score_for_sensors_timings(
    t1: f64,
    t2: f64,
    t3: f64,
    t4: f64,
) -> Result<(Point, u32), String> {
    // Create the `ImpactLocator` using the provided sensor timings and default sensor positions.
    let sensors = default_sensors();
    let locator = ImpactLocator::new(sensors, [t1, t2, t3, t4]);

    println!(
        "Received imapcts delays (us): t1={}, t2={}, t3={}, t4={}",
        t1, t2, t3, t4
    );

    // Use the `ImpactLocator` to find the impact point based on the provided sensor timings.
    let reached_impact = locator
        .locate()
        .ok_or_else(|| String::from("solver did not find an impact point"))?;

    // Calculate the score using the `Score` struct based on the reached impact point.
    let mut score = Score::new(reached_impact);

    Ok((reached_impact, score.get_score()))
}
