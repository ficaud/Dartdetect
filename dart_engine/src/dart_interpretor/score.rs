use super::multiplicator::multiplier_from_radius;
use super::zone::sector_from_angle;
use crate::dart_core::Point;
use crate::dart_game::game::ShotResult;

const BOARD_CENTER_X: f64 = 250.0;
const BOARD_CENTER_Y: f64 = 250.0;
const DOUBLE_BULL_RADIUS: f64 = 6.35;
const SINGLE_BULL_RADIUS: f64 = 15.9;

/// Represents the score of a single dart throw,
/// including the points scored and the coordinates of the impact on the board
pub struct Score {
    pub points: u32,
    pub impact_coordinates: Point,
}

/// Score creation and calculation logic based on the impact coordinates on the dartboard.
impl Score {
    /// Create a new Score instance with the given impact coordinates.
    ///
    /// # Arguments
    /// - impact_coordinates: A `Point` struct representing the x and y coordinates of the dart's impact on the board.
    ///
    /// This function initializes a `Score` instance with the provided impact coordinates and a default points.
    pub fn new(impact_coordinates: Point) -> Self {
        Score {
            points: 0,
            impact_coordinates,
        }
    }

    /// Calculate the angle of the impact point relative to the center of the board.
    ///
    /// This function returns the angle in degrees, measured clockwise from the top of the board.
    fn board_angle(&self) -> f64 {
        let centered = self.centered_coordinates();
        let math_angle = centered.y.atan2(centered.x).to_degrees();

        (90.0 - math_angle).rem_euclid(360.0)
    }

    /// Convert the impact coordinates to a centered coordinate system with the origin at the center of the board.
    ///
    /// This function returns a `Point` struct representing the coordinates of the impact relative to the center of the board.
    fn centered_coordinates(&self) -> Point {
        Point::new(
            self.impact_coordinates.x - BOARD_CENTER_X,
            self.impact_coordinates.y - BOARD_CENTER_Y,
        )
    }

    /// Calculate the score based on the impact coordinates.
    ///
    /// This function determines the score by calculating the distance from the center of the board and the
    /// angle of the impact point, then applying the appropriate multipliers and sector values to compute the final score.
    pub fn get_score(&mut self) -> u32 {
        let centered = self.centered_coordinates();
        let radius = (centered.x.powi(2) + centered.y.powi(2)).sqrt();

        if radius <= DOUBLE_BULL_RADIUS {
            self.points = 50;
            return self.points;
        }

        if radius <= SINGLE_BULL_RADIUS {
            self.points = 25;
            return self.points;
        }

        let multiplier = multiplier_from_radius(radius as f32);
        let zone = sector_from_angle(self.board_angle());

        self.points = multiplier * u32::from(zone);

        self.points
    }

    /// Compute a `ShotResult` (sector + multiplier) from the impact coordinates.
    ///
    /// This is the bridge between the score interpreter and the game engine:
    /// the returned `ShotResult` can be fed directly into `GameSession::apply_shot()`.
    pub fn get_shortresult(&mut self) -> ShotResult {
        let centered = self.centered_coordinates();
        let radius = (centered.x.powi(2) + centered.y.powi(2)).sqrt();

        if radius <= DOUBLE_BULL_RADIUS {
            self.points = 50;
            return ShotResult {
                sector: 25,
                multiplier: 2,
            };
        }

        if radius <= SINGLE_BULL_RADIUS {
            self.points = 25;
            return ShotResult {
                sector: 25,
                multiplier: 1,
            };
        }

        let multiplier = multiplier_from_radius(radius as f32);
        let zone = sector_from_angle(self.board_angle());

        self.points = multiplier * u32::from(zone);

        ShotResult {
            sector: zone as u32,
            multiplier,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Score;
    use crate::dart_core::Point;
    use crate::dart_game::game::ShotResult;

    #[test]
    fn double_bull_scores_fifty() {
        let mut score = Score::new(Point::new(250.0, 250.0));

        assert_eq!(score.get_score(), 50);
    }

    #[test]
    fn single_bull_scores_twenty_five() {
        let mut score = Score::new(Point::new(260.0, 250.0));

        assert_eq!(score.get_score(), 25);
    }

    #[test]
    fn top_single_scores_twenty() {
        let mut score = Score::new(Point::new(250.0, 300.0));

        assert_eq!(score.get_score(), 20);
    }

    #[test]
    fn right_triple_scores_eighteen() {
        let mut score = Score::new(Point::new(350.0, 250.0));

        assert_eq!(score.get_score(), 18);
    }

    #[test]
    fn outside_board_scores_zero() {
        let mut score = Score::new(Point::new(480.0, 250.0));

        assert_eq!(score.get_score(), 0);
    }

    // ─── get_shortresult tests ──────────────────────

    #[test]
    fn double_bull_shotresult() {
        let mut score = Score::new(Point::new(250.0, 250.0));
        let result = score.get_shortresult();

        assert_eq!(
            result,
            ShotResult {
                sector: 25,
                multiplier: 2
            }
        );
        assert_eq!(score.points, 50);
    }

    #[test]
    fn single_bull_shotresult() {
        let mut score = Score::new(Point::new(260.0, 250.0));
        let result = score.get_shortresult();

        assert_eq!(
            result,
            ShotResult {
                sector: 25,
                multiplier: 1
            }
        );
        assert_eq!(score.points, 25);
    }

    #[test]
    fn top_single_shotresult() {
        let mut score = Score::new(Point::new(250.0, 300.0));
        let result = score.get_shortresult();

        assert_eq!(
            result,
            ShotResult {
                sector: 20,
                multiplier: 1
            }
        );
        assert_eq!(score.points, 20);
    }

    #[test]
    fn right_triple_shotresult() {
        let mut score = Score::new(Point::new(350.0, 250.0));
        let result = score.get_shortresult();

        assert_eq!(
            result,
            ShotResult {
                sector: 6,
                multiplier: 3
            }
        );
        assert_eq!(score.points, 18);
    }

    #[test]
    fn outside_board_shotresult() {
        let mut score = Score::new(Point::new(480.0, 250.0));
        let result = score.get_shortresult();

        assert_eq!(
            result,
            ShotResult {
                sector: 6,
                multiplier: 0
            }
        );
        assert_eq!(score.points, 0);
    }
}
