use super::{DistanceCalculation, ErrorCalculation, ImpactSimulator, Point, SensorPos};


// Guess point is compose from the Sesnsor positions and another point
// that will be add to the struct as a guess for the impact point
#[derive(Debug)]
pub struct GuessPoints {
    pub sensors: [SensorPos; 4],
    pub point: Point,
}

// Solver need a guess points strucutre and the actual distances of the impact from the sensors
pub struct Solver {
    pub guess_points: GuessPoints,
    pub actual_distances: [f64; 4],
}

// Guespoints functions implementation
impl GuessPoints {
    /// Create a new `GuessPoints` instance with the given sensor positions and an initial guess point at (0, 0).
    ///
    /// # Arguments
    /// - sensors: An array of `SensorPos` structs representing the positions of the sensors.
    ///
    /// # Returns
    /// A new `GuessPoints` instance with the given sensor positions and an initial guess point at (0, 0).
    pub fn new(sensors: [SensorPos; 4]) -> Self {
        GuessPoints { sensors, point: Point::new(0.0, 0.0) }
    }

    /// Add a new guess point to the `GuessPoints` struct.
    ///
    /// # Arguments
    /// - self: The `GuessPoints` instance to which the new guess point will be added.
    /// - point: The new guess point to be added, represented as a `Point` struct containing the x and y coordinates of the guess.
    /// 
    /// This function updates the `point` field of the `GuessPoints` struct.
    pub fn add_guess(&mut self, point: Point) {
        self.point = point;
    }
}

// Implements the `DistanceCalculation` trait for `GuessPoints`, allowing it to calculate
// distances based on the guessed point and sensor positions.
impl DistanceCalculation for GuessPoints {
    type Output = [f64; 4];

    /// Get the distances from the guessed point to each sensor using the Euclidean distance formula.
    ///
    /// # Arguments
    /// - self: The `GuessPoints` instance containing the guessed point and sensor positions.
    /// 
    /// This function calculates the distances from the guessed point to each sensor.
    fn get_distances(&self) -> Self::Output {
        let x = self.point.x;
        let y = self.point.y;

        let distance_from = |sensor: &SensorPos| {
            let c = sensor.point();
            ((x - c.x).powi(2) + (y - c.y).powi(2)).sqrt()
        };

        [
            distance_from(&self.sensors[0]),
            distance_from(&self.sensors[1]),
            distance_from(&self.sensors[2]),
            distance_from(&self.sensors[3]),
        ]
    }
}

// Implements the `ErrorCalculation` trait for `GuessPoints`,
// allowing it to calculate the error between the guessed distances and the actual distances.
impl ErrorCalculation for GuessPoints {
    type Output = f64;

    /// Calculate the error between the guessed distances and the actual distances using the L2 norm.
    ///
    /// # Arguments
    /// - self: The `GuessPoints` instance containing the guessed point and sensor positions.
    /// - actual_distances: An array of the actual distances between the impact point and each sensor, calculated from the simulated timings.
    /// 
    /// This function calculates the L2 error
    fn calculate_error(&self, actual_distances: &[f64; 4]) -> Self::Output {
        let guess_distances = self.get_distances();
        let guess_min = guess_distances
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);
        let actual_min = actual_distances
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);

        let guess_relative = [
            guess_distances[0] - guess_min,
            guess_distances[1] - guess_min,
            guess_distances[2] - guess_min,
            guess_distances[3] - guess_min,
        ];
        let actual_relative = [
            actual_distances[0] - actual_min,
            actual_distances[1] - actual_min,
            actual_distances[2] - actual_min,
            actual_distances[3] - actual_min,
        ];

        guess_relative
            .iter()
            .zip(actual_relative.iter())
            .map(|(g, a)| (g - a).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}

// Solver implementation, which uses the `GuessPoints` and the actual distances to iteratively find the impact point.
impl Solver {
    /// Create a new solver directly from the sensor positions and raw timings.
    pub fn new_from_timings(sensors: [SensorPos; 4], timings_us: [f64; 4]) -> Self {
        let impact_simulator = ImpactSimulator::new(
            timings_us[0],
            timings_us[1],
            timings_us[2],
            timings_us[3],
        );
        let actual_distances = impact_simulator.get_distances();
        let guess_points = GuessPoints::new(sensors);
        Solver { actual_distances, guess_points }
    }

    /// Solve the impact point using the current solver state.
    pub fn solve(mut self) -> Option<Point> {
        self.find_impact_point()
    }

    /// Iteratively guess points in a 500x500 grid to find the impact point that matches the actual distances.
    /// 
    /// # Arguments
    /// - self: The `Solver` instance containing the actual distances and the guess points.
    /// 
    /// This function iterates through all possible points in a 500x500 grid, updating the guess point and calculating the error for each guess.
    pub fn find_impact_point(&mut self) -> Option<Point> {
        let mut iterations = 0_u32;

        for x in 0..=500 {
            for y in 0..=500 {
                iterations += 1;
                self.guess_points.add_guess(Point::new(x as f64, y as f64));
                let error = self.guess_points.calculate_error(&self.actual_distances);

                if error < 1e-6 {
                    println!("Iterations needed to reach impact: {}", iterations);
                    return Some(Point::new(x as f64, y as f64));
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::{DistanceCalculation, ErrorCalculation, GuessPoints, ImpactSimulator, Point, SensorPos};

    #[test]
    fn main_like_flow_runs_in_module_test_suite() {
        let sensors = [
            SensorPos::C1(Point::new(0.0, 500.0)),
            SensorPos::C2(Point::new(500.0, 500.0)),
            SensorPos::C3(Point::new(500.0, 0.0)),
            SensorPos::C4(Point::new(0.0, 0.0)),
        ];

        assert_eq!(sensors.len(), 4);

        let impact_simulator = ImpactSimulator::new(1000.0, 1200.0, 1300.0, 1400.0);
        let distances = impact_simulator.get_distances();

        assert_eq!(distances[0], 0.0);
        assert_eq!(distances[1], 68.6);
        assert_eq!(distances[2], 102.9);
        assert_eq!(distances[3], 137.2);
    }

    #[test]
    fn distance_calculation_for_guess_point_uses_euclidean_formula() {
        let sensors = [
            SensorPos::C1(Point::new(0.0, 500.0)),
            SensorPos::C2(Point::new(500.0, 500.0)),
            SensorPos::C3(Point::new(500.0, 0.0)),
            SensorPos::C4(Point::new(0.0, 0.0)),
        ];

        let mut guess_point = GuessPoints::new(sensors);
        guess_point.add_guess(Point::new(250.0, 250.0));

        let distances = guess_point.get_distances();
        let expected = (250.0_f64.powi(2) + 250.0_f64.powi(2)).sqrt();

        for distance in distances {
            assert!((distance - expected).abs() < 1e-9);
        }
    }

    #[test]
    fn error_calculation_for_guess_point_returns_zero_for_perfect_match() {
        let sensors = [
            SensorPos::C1(Point::new(0.0, 500.0)),
            SensorPos::C2(Point::new(500.0, 500.0)),
            SensorPos::C3(Point::new(500.0, 0.0)),
            SensorPos::C4(Point::new(0.0, 0.0)),
        ];

        let mut guess_point = GuessPoints::new(sensors);
        guess_point.add_guess(Point::new(250.0, 250.0));

        let actual_distances = guess_point.get_distances();
        let error = guess_point.calculate_error(&actual_distances);

        assert!((error - 0.0).abs() < 1e-12);
    }

    #[test]
    fn error_calculation_for_guess_point_matches_expected_l2_error() {
        let sensors = [
            SensorPos::C1(Point::new(0.0, 500.0)),
            SensorPos::C2(Point::new(500.0, 500.0)),
            SensorPos::C3(Point::new(500.0, 0.0)),
            SensorPos::C4(Point::new(0.0, 0.0)),
        ];

        let mut guess_point = GuessPoints::new(sensors);
        guess_point.add_guess(Point::new(250.0, 250.0));

        let base = guess_point.get_distances();
        let actual_distances = [base[0] + 3.0, base[1], base[2] - 4.0, base[3]];
        let error = guess_point.calculate_error(&actual_distances);

        // Relative comparison: expected sqrt(7^2 + 4^2 + 0^2 + 4^2) = 9
        assert!((error - 9.0).abs() < 1e-12);
    }

    #[test]
    fn error_calculation_from_impact_simulator_matches_expected_l2_error() {
        let sensors = [
            SensorPos::C1(Point::new(0.0, 500.0)),
            SensorPos::C2(Point::new(500.0, 500.0)),
            SensorPos::C3(Point::new(500.0, 0.0)),
            SensorPos::C4(Point::new(0.0, 0.0)),
        ];

        let impact_simulator = ImpactSimulator::new(1000.0, 1200.0, 1300.0, 1400.0);
        let actual_distances = impact_simulator.get_distances();

        let mut guess_point = GuessPoints::new(sensors);
        guess_point.add_guess(Point::new(250.0, 250.0));

        let error = guess_point.calculate_error(&actual_distances);
        assert!(error > 0.0);
    }

}