use crate::dart_core::Point;

use super::{DistanceCalculation, ErrorCalculation, SensorPos};

pub struct Gradient {
    pub sensors: [SensorPos; 4],
    pub prev_error: f64,
    pub alpha: f64,
    pub old_x: f64,
    pub old_y: f64,
    pub x_epsilon: Point,
    pub y_epsilon: Point,
    pub epsilon: f64,
}

impl Gradient {
    pub fn new(alpha: f64, epsilon: f64, sensors: [SensorPos; 4]) -> Self {
        Gradient {
            sensors,
            prev_error: f64::MAX,
            alpha,
            old_x: 0.0,
            old_y: 0.0,
            x_epsilon: Point::new(0.0, 0.0),
            y_epsilon: Point::new(0.0, 0.0),
            epsilon,
        }
    }

    pub fn get_alpha(&self) -> f64 {
        self.alpha
    }

    pub fn set_alpha(&mut self, alpha: f64) {
        self.alpha = alpha;
    }

    pub fn _get_epsilon(&self) -> f64 {
        self.epsilon
    }

    pub fn _set_epsilon(&mut self, epsilon: f64) {
        self.epsilon = epsilon;
    }

    pub fn _set_probe_point(&mut self, x: f64, y: f64) {
        let point = Point::new(x, y);
        self.x_epsilon = point;
        self.y_epsilon = point;
    }

    pub fn get_new_guess(
        &mut self,
        actual_distances: &[f64; 4],
        old_x: f64,
        old_y: f64,
    ) -> (f64, f64) {
        self.old_x = old_x;
        self.old_y = old_y;
        self.prev_error = self.calculate_error(Point::new(old_x, old_y), actual_distances);
        self.x_epsilon = Point::new(old_x + self.epsilon, old_y);
        self.y_epsilon = Point::new(old_x, old_y + self.epsilon);

        let error_x = self.calculate_error(self.x_epsilon, actual_distances);
        let error_y = self.calculate_error(self.y_epsilon, actual_distances);

        (error_x, error_y)
    }

    pub fn calculate_gradient(
        &mut self,
        actual_distances: &[f64; 4],
        old_x: f64,
        old_y: f64,
    ) -> (f64, f64) {
        let (error_x, error_y) = self.get_new_guess(actual_distances, old_x, old_y);

        let gradient_x = (error_x - self.prev_error) / self.epsilon;
        let gradient_y = (error_y - self.prev_error) / self.epsilon;

        (gradient_x, gradient_y)
    }
}

impl ErrorCalculation for Gradient {
    type Output = f64;

    fn calculate_error(&self, point: Point, actual_distances: &[f64; 4]) -> Self::Output {
        let guess_distances = self.get_distances(point);
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

impl DistanceCalculation for Gradient {
    type Output = [f64; 4];

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

#[cfg(test)]
mod tests {
    use super::{DistanceCalculation, ErrorCalculation, Gradient, SensorPos};
    use crate::dart_calculator::ImpactSimulator;
    use crate::dart_core::Point;

    #[test]
    fn gradient_error_is_zero_for_matching_probe_point() {
        let sensors = [
            SensorPos::C1(Point::new(0.0, 500.0)),
            SensorPos::C2(Point::new(500.0, 500.0)),
            SensorPos::C3(Point::new(500.0, 0.0)),
            SensorPos::C4(Point::new(0.0, 0.0)),
        ];

        let mut gradient = Gradient::new(0.1, 1.0, sensors);
        gradient._set_probe_point(250.0, 250.0);

        let actual_distances = gradient.get_distances(gradient.x_epsilon);
        let error = gradient.calculate_error(gradient.x_epsilon, &actual_distances);

        assert!((error - 0.0).abs() < 1e-12);
    }

    #[test]
    fn get_new_guess_highlights_direction_for_a_better_second_guess() {
        let sensors = [
            SensorPos::C1(Point::new(0.0, 500.0)),
            SensorPos::C2(Point::new(500.0, 500.0)),
            SensorPos::C3(Point::new(500.0, 0.0)),
            SensorPos::C4(Point::new(0.0, 0.0)),
        ];

        let first_guess = Point::new(100.0, 250.0);
        let target_point = Point::new(400.0, 250.0);
        let baseline_time_us = 1_000.0;
        let actual_distances =
            ImpactSimulator::from_point(&target_point, &sensors, baseline_time_us)
                .get_distances(Point::new(0.0, 0.0));

        let mut gradient = Gradient::new(0.1, 20.0, sensors);

        gradient._set_probe_point(first_guess.x, first_guess.y);
        let base_error = gradient.calculate_error(first_guess, &actual_distances);
        let (gradient_x, gradient_y) =
            gradient.calculate_gradient(&actual_distances, first_guess.x, first_guess.y);
        let error_x = gradient.calculate_error(gradient.x_epsilon, &actual_distances);
        let error_y = gradient.calculate_error(gradient.y_epsilon, &actual_distances);

        assert_eq!(gradient.x_epsilon, Point::new(120.0, 250.0));
        assert_eq!(gradient.y_epsilon, Point::new(100.0, 270.0));
        assert!((gradient.prev_error - base_error).abs() < 1e-12);
        assert!(error_x < base_error);
        assert!(error_y > base_error);
        assert!(error_x < error_y);
        assert!(gradient_x < 0.0);
        assert!(gradient_y > 0.0);
    }
}
