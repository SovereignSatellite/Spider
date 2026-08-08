#![crate_type = "cdylib"]

// Adapted from nbody-rust-1 under ../LICENSE.txt.

const BODY_COUNT: usize = 5;
const DAYS_PER_YEAR: f64 = 365.24;
const DELTA_TIME: f64 = 0.01;
const SOLAR_MASS: f64 = 4.0 * core::f64::consts::PI * core::f64::consts::PI;

#[derive(Clone, Copy)]
struct Body {
	position_x: f64,
	position_y: f64,
	position_z: f64,
	velocity_x: f64,
	velocity_y: f64,
	velocity_z: f64,
	mass: f64,
}

const INITIAL_BODIES: [Body; BODY_COUNT] = [
	Body {
		position_x: 0.0,
		position_y: 0.0,
		position_z: 0.0,
		velocity_x: 0.0,
		velocity_y: 0.0,
		velocity_z: 0.0,
		mass: SOLAR_MASS,
	},
	Body {
		position_x: 4.841_431_442_464_721,
		position_y: -1.160_320_044_027_428_4,
		position_z: -0.103_622_044_471_123_11,
		velocity_x: 0.001_660_076_642_744_037 * DAYS_PER_YEAR,
		velocity_y: 0.007_699_011_184_197_404 * DAYS_PER_YEAR,
		velocity_z: -0.000_069_046_001_697_206_3 * DAYS_PER_YEAR,
		mass: 0.000_954_791_938_424_326_6 * SOLAR_MASS,
	},
	Body {
		position_x: 8.343_366_718_244_58,
		position_y: 4.124_798_564_124_305,
		position_z: -0.403_523_417_114_321_4,
		velocity_x: -0.002_767_425_107_268_624 * DAYS_PER_YEAR,
		velocity_y: 0.004_998_528_012_349_172 * DAYS_PER_YEAR,
		velocity_z: 0.000_023_041_729_757_376_393 * DAYS_PER_YEAR,
		mass: 0.000_285_885_980_666_130_8 * SOLAR_MASS,
	},
	Body {
		position_x: 12.894_369_562_139_131,
		position_y: -15.111_151_401_698_631,
		position_z: -0.223_307_578_892_655_73,
		velocity_x: 0.002_964_601_375_647_616 * DAYS_PER_YEAR,
		velocity_y: 0.002_378_471_739_594_809_5 * DAYS_PER_YEAR,
		velocity_z: -0.000_029_658_956_854_023_756 * DAYS_PER_YEAR,
		mass: 0.000_043_662_440_433_515_63 * SOLAR_MASS,
	},
	Body {
		position_x: 15.379_697_114_850_911,
		position_y: -25.919_314_609_987_964,
		position_z: 0.179_258_772_950_371_18,
		velocity_x: 0.002_680_677_724_903_893_2 * DAYS_PER_YEAR,
		velocity_y: 0.001_628_241_700_382_423 * DAYS_PER_YEAR,
		velocity_z: -0.000_095_159_225_451_971_59 * DAYS_PER_YEAR,
		mass: 0.000_051_513_890_204_661_145 * SOLAR_MASS,
	},
];

fn offset_momentum(bodies: &mut [Body; BODY_COUNT]) {
	let mut momentum_x = 0.0;
	let mut momentum_y = 0.0;
	let mut momentum_z = 0.0;

	for body in bodies.iter() {
		momentum_x += body.velocity_x * body.mass;
		momentum_y += body.velocity_y * body.mass;
		momentum_z += body.velocity_z * body.mass;
	}

	bodies[0].velocity_x = -momentum_x / SOLAR_MASS;
	bodies[0].velocity_y = -momentum_y / SOLAR_MASS;
	bodies[0].velocity_z = -momentum_z / SOLAR_MASS;
}

fn advance(bodies: &mut [Body; BODY_COUNT], step_count: u32) {
	for _ in 0..step_count {
		let mut remaining_bodies = &mut bodies[..];
		while let Some((body, later_bodies)) = remaining_bodies.split_first_mut() {
			remaining_bodies = later_bodies;

			for other_body in remaining_bodies.iter_mut() {
				let distance_x = body.position_x - other_body.position_x;
				let distance_y = body.position_y - other_body.position_y;
				let distance_z = body.position_z - other_body.position_z;
				let squared_distance =
					distance_x * distance_x + distance_y * distance_y + distance_z * distance_z;
				let magnitude = DELTA_TIME / (squared_distance * squared_distance.sqrt());

				let other_magnitude = other_body.mass * magnitude;
				body.velocity_x -= distance_x * other_magnitude;
				body.velocity_y -= distance_y * other_magnitude;
				body.velocity_z -= distance_z * other_magnitude;

				let body_magnitude = body.mass * magnitude;
				other_body.velocity_x += distance_x * body_magnitude;
				other_body.velocity_y += distance_y * body_magnitude;
				other_body.velocity_z += distance_z * body_magnitude;
			}

			body.position_x += DELTA_TIME * body.velocity_x;
			body.position_y += DELTA_TIME * body.velocity_y;
			body.position_z += DELTA_TIME * body.velocity_z;
		}
	}
}

fn energy(bodies: &[Body; BODY_COUNT]) -> f64 {
	let mut total_energy = 0.0;

	for (body_index, body) in bodies.iter().enumerate() {
		let squared_velocity = body.velocity_x * body.velocity_x
			+ body.velocity_y * body.velocity_y
			+ body.velocity_z * body.velocity_z;
		total_energy += squared_velocity * body.mass / 2.0;

		for other_body in &bodies[body_index + 1..] {
			let distance_x = body.position_x - other_body.position_x;
			let distance_y = body.position_y - other_body.position_y;
			let distance_z = body.position_z - other_body.position_z;
			let distance =
				(distance_x * distance_x + distance_y * distance_y + distance_z * distance_z)
					.sqrt();
			total_energy -= body.mass * other_body.mass / distance;
		}
	}

	total_energy
}

#[unsafe(export_name = "benchmark")]
pub extern "C" fn benchmark(step_count: u32) -> f64 {
	let mut bodies = INITIAL_BODIES;
	offset_momentum(&mut bodies);
	core::hint::black_box(energy(&bodies));
	advance(&mut bodies, step_count);
	energy(&bodies)
}

#[cfg(test)]
mod tests {
	#[test]
	fn computes_known_result() {
		let difference = (super::benchmark(1_000) - -0.169_087_605).abs();
		assert!(difference < 0.000_000_001);
	}
}
