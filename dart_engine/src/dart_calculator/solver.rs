use super::{DistanceCalculation, ErrorCalculation, Gradient, ImpactSimulator, Point, SensorPos};

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
        GuessPoints {
            sensors,
            point: Point::new(0.0, 0.0),
        }
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
    fn get_distances(&self, point: Point) -> Self::Output {
        let x = point.x;
        let y = point.y;

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
    fn calculate_error(&self, point: Point, actual_distances: &[f64; 4]) -> Self::Output {
        let x = point.x;
        let y = point.y;

        let distance_from = |sensor: &SensorPos| {
            let c = sensor.point();
            ((x - c.x).powi(2) + (y - c.y).powi(2)).sqrt()
        };

        let guess_distances = [
            distance_from(&self.sensors[0]),
            distance_from(&self.sensors[1]),
            distance_from(&self.sensors[2]),
            distance_from(&self.sensors[3]),
        ];
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
    /// Calculate the error at a given point by updating the guess and comparing it to the actual distances.
    ///
    /// # Arguments
    /// - self: The `Solver` instance containing the guess points and actual distances.
    /// - point: The point at which to calculate the error, represented as a `Point` struct containing the x and y coordinates of the point.
    ///
    /// This function updates the guess point with the provided point and calculates the error using the `calculate_error` method of the `GuessPoints` struct.
    fn error_at(&mut self, point: Point) -> f64 {
        self.guess_points.add_guess(point);
        self.guess_points
            .calculate_error(self.guess_points.point, &self.actual_distances)
    }

    /// Search for integer candidate points around the given center point within a specified radius and return the one with the lowest error.
    ///
    /// # Arguments
    /// - self: The `Solver` instance containing the guess points and actual distances.
    /// - center: The center point around which to search for integer candidate points, represented as a `Point` struct containing the x and y coordinates of the center.
    /// - radius: The radius within which to search for integer candidate points, represented as an `i32` value.
    ///
    /// This function iterates through integer candidate points around the center point within the specified radius, calculates the error for each candidate point using the `error_at` method,
    /// and returns the candidate point with the lowest error if it is below a certain threshold.
    fn snap_to_integer_candidate(&mut self, center: Point, radius: i32) -> Option<Point> {
        let center_x = center.x.round() as i32;
        let center_y = center.y.round() as i32;
        let mut best_point = None;
        let mut best_error = f64::INFINITY;

        for dx in -radius..=radius {
            for dy in -radius..=radius {
                let candidate = Point::new(
                    (center_x + dx).clamp(0, 500) as f64,
                    (center_y + dy).clamp(0, 500) as f64,
                );
                let error = self.error_at(candidate);

                if error < best_error {
                    best_error = error;
                    best_point = Some(candidate);
                }
            }
        }

        if best_error < 1e-6 { best_point } else { None }
    }

    /// Create a new solver directly from the sensor positions and raw timings.
    pub fn new_from_timings(sensors: [SensorPos; 4], timings_us: [f64; 4]) -> Self {
        let impact_simulator =
            ImpactSimulator::new(timings_us[0], timings_us[1], timings_us[2], timings_us[3]);
        let actual_distances = impact_simulator.get_distances(Point::new(0.0, 0.0));
        let guess_points = GuessPoints::new(sensors);
        Solver {
            actual_distances,
            guess_points,
        }
    }

    /// Solve the impact point using the current solver state.
    pub fn solve(mut self) -> Option<Point> {
        self.find_impact_point()
    }

    /// Iteratively update the guess point with gradient descent to find the impact point.
    ///
    /// # Arguments
    /// - self: The `Solver` instance containing the actual distances and the guess points.
    ///
    /// This function updates the current guess with the local gradient.
    /// It uses several starts and a local integer snap search to keep the solver in gradient-only mode.
    pub fn find_impact_point(&mut self) -> Option<Point> {
        let mut iterations = 0_u32;
        let mut gradient = Gradient::new(0.5, 1.0, self.guess_points.sensors);

        // Try multiple starting points to increase the chances of finding the global minimum and avoid local minima.
        let starting_points = [
            Point::new(250.0, 250.0),
            Point::new(125.0, 125.0),
            Point::new(125.0, 375.0),
            Point::new(375.0, 125.0),
            Point::new(375.0, 375.0),
            Point::new(250.0, 125.0),
            Point::new(250.0, 375.0),
            Point::new(125.0, 250.0),
            Point::new(375.0, 250.0),
            Point::new(0.0, 0.0),
            Point::new(0.0, 500.0),
            Point::new(500.0, 0.0),
            Point::new(500.0, 500.0),
            Point::new(250.0, 0.0),
            Point::new(250.0, 500.0),
            Point::new(0.0, 250.0),
            Point::new(500.0, 250.0),
        ];
        // Limit the number of iterations per starting point to prevent infinite loops in case of convergence issues.
        let max_iterations_per_start = 5_000_u32;

        for start in starting_points {
            // Init guess for the given starting point accross the board
            let mut current_guess = start;
            let mut current_error = self.error_at(current_guess);
            gradient.set_alpha(2.0);

            for _ in 0..max_iterations_per_start {
                iterations += 1;

                // Check if the current error is low enough to consider the impact point found
                if current_error < 1e-6 {
                    println!("Iterations needed to reach impact: {}", iterations);
                    return Some(Point::new(current_guess.x.round(), current_guess.y.round()));
                }

                // Check if snapping to an integer candidate point results in a low enough error to consider the impact point found
                if let Some(point) = self.snap_to_integer_candidate(current_guess, 2) {
                    println!("Iterations needed to reach impact: {}", iterations);
                    return Some(point);
                }

                // Calculate the gradient at the current guess point and update the guess point in the direction of the negative gradient.
                let (gradient_x, gradient_y) = gradient.calculate_gradient(
                    &self.actual_distances,
                    current_guess.x,
                    current_guess.y,
                );

                let gradient_norm = (gradient_x.powi(2) + gradient_y.powi(2)).sqrt();
                if gradient_norm < 1e-12 {
                    break;
                }

                // Adaptive learning rate: If the error does not decrease, reduce the learning rate and try again.
                // If it does decrease, slightly increase the learning rate for the next iteration to speed up convergence.
                let mut learning_rate = gradient.get_alpha();
                let mut accepted_next_guess = None;

                for _ in 0..12 {
                    // Calculate the candidate next guess.
                    let candidate = Point::new(
                        (current_guess.x - (learning_rate * gradient_x)).clamp(0.0, 500.0),
                        (current_guess.y - (learning_rate * gradient_y)).clamp(0.0, 500.0),
                    );

                    // If the candidate guess is too close to the current guess, reduce the learning rate and try again to avoid getting stuck in a local minimum or making negligible updates.
                    if (candidate.x - current_guess.x).abs() < 1e-9
                        && (candidate.y - current_guess.y).abs() < 1e-9
                    {
                        learning_rate *= 0.5;
                        continue;
                    }

                    // Calculate the error for the candidate guess and check if it is an improvement over the current error.
                    // If it is, accept the candidate guess and break out of the learning rate adjustment loop.
                    let candidate_error = self.error_at(candidate);

                    if candidate_error + 1e-12 < current_error {
                        accepted_next_guess = Some((candidate, candidate_error, learning_rate));
                        break;
                    }

                    // If the candidate guess is not an improvement, reduce the learning rate and try again to find a better candidate guess.
                    learning_rate *= 0.5;
                }

                // If no candidate guess was accepted after adjusting the learning rate, break out of the iteration loop to try a new starting point.
                let Some((next_guess, next_error, accepted_alpha)) = accepted_next_guess else {
                    break;
                };

                // Update the learning rate based on whether the candidate guess was accepted or not, and update the current guess and error for the next iteration.
                gradient.set_alpha((accepted_alpha * 1.2).min(8.0));
                current_guess = next_guess;
                current_error = next_error;
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DistanceCalculation, ErrorCalculation, GuessPoints, ImpactSimulator, Point, SensorPos,
    };

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
        let distances = impact_simulator.get_distances(Point::new(0.0, 0.0));

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

        let distances = guess_point.get_distances(guess_point.point);
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

        let actual_distances = guess_point.get_distances(guess_point.point);
        let error = guess_point.calculate_error(guess_point.point, &actual_distances);

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

        let base = guess_point.get_distances(guess_point.point);
        let actual_distances = [base[0] + 3.0, base[1], base[2] - 4.0, base[3]];
        let error = guess_point.calculate_error(guess_point.point, &actual_distances);

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
        let actual_distances = impact_simulator.get_distances(Point::new(0.0, 0.0));

        let mut guess_point = GuessPoints::new(sensors);
        guess_point.add_guess(Point::new(250.0, 250.0));

        let error = guess_point.calculate_error(guess_point.point, &actual_distances);
        assert!(error > 0.0);
    }
}
