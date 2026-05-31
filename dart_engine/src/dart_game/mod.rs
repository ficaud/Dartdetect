use serde::Serialize;
use std::time::Instant;

use crate::dart_calculator::{ImpactLocator, ImpactSimulator, Point, SensorPos};
use crate::dart_interpretor::score::Score;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ScorePayload {
	pub impact_x: f64,
	pub impact_y: f64,
	pub score: u32,
	pub processing_time_us: u128,
}

const BOARD_SIZE: f64 = 500.0;

fn default_sensors() -> [SensorPos; 4] {
	[
		SensorPos::C1(Point::new(0.0, BOARD_SIZE)),
		SensorPos::C2(Point::new(BOARD_SIZE, BOARD_SIZE)),
		SensorPos::C3(Point::new(BOARD_SIZE, 0.0)),
		SensorPos::C4(Point::new(0.0, 0.0)),
	]
}

pub fn calculate_score_for_impact_point(impact_point: Point) -> Result<ScorePayload, String> {
	let sensors = default_sensors();
	let processing_start = Instant::now();
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

	let reached_impact = locator
		.locate()
		.ok_or_else(|| String::from("solver did not find an impact point"))?;

	let mut score = Score::new(reached_impact);
	Ok(build_score_payload(reached_impact, &mut score, processing_start))
}

fn build_score_payload(
	impact_point: Point,
	score: &mut Score,
	processing_start: Instant,
) -> ScorePayload {
	ScorePayload {
		impact_x: impact_point.x,
		impact_y: impact_point.y,
		score: score.get_score(),
		processing_time_us: processing_start.elapsed().as_micros(),
	}
}