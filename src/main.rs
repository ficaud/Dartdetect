mod dart_core;
mod dart_calculator;
use crate::dart_calculator::*;
mod dart_interpretor;
use crate::dart_interpretor::score::Score;

fn main() {

    // =================================================================
    // Step 1: Initialize the sensors and the impact point (simulation)
    // =================================================================
    
    // Init the sensors position
    let sensors = [
        SensorPos::C1(Point::new(0.0, 500.0)),
        SensorPos::C2(Point::new(500.0, 500.0)),
        SensorPos::C3(Point::new(500.0, 0.0)),
        SensorPos::C4(Point::new(0.0, 0.0)),
    ];

    let impact_point: Point = Point::new(250.0, 420.0);

    // Simulate the impact and get the actual distances from the sensors
    let baseline_time_us = 1_000.0;
    let impact_simulator = ImpactSimulator::from_point(&impact_point, &sensors, baseline_time_us);
    
    // =================================================================
    // Step 2: Locate the impact point using the sensors and the simulated timings
    // =================================================================
    
    // locate the impact point using the sensors and the simulated timings
    let locator = ImpactLocator::new(
        sensors,
        [
            impact_simulator.t1,
            impact_simulator.t2,
            impact_simulator.t3,
            impact_simulator.t4,
        ],
    );

    // Get the reached impact point from the locator
    let reached_impact = locator.locate();

    println!("Reached impact point: {:?}", reached_impact);


    match reached_impact {
        Some(point) => {
            assert_eq!(point.x, impact_point.x);
            assert_eq!(point.y, impact_point.y);
        }
        None => panic!("solver did not find an impact point"),
    }

    // =================================================================
    // Step 3: Estimate the amount of points scored
    // =================================================================
    let point = reached_impact.expect("solver did not find an impact point");
    let mut score = Score::new(point);

    // =================================================================
    // Step 5 : TODO - move on the to rest of the game logic
    // =================================================================

    println!("Score: {}", score.get_score());
}
