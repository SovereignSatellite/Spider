#![crate_type = "cdylib"]

// Adapted from spectralnorm-rust-4 under ../LICENSE.txt.

const ITERATION_COUNT: usize = 10;
const MAXIMUM_VECTOR_LENGTH: usize = 8_192;
const MAXIMUM_VECTOR_PAIR_COUNT: usize = MAXIMUM_VECTOR_LENGTH / 2;

fn matrix_denominators(rows: [usize; 2], columns: [usize; 2]) -> [f64; 2] {
	[
		((rows[0] + columns[0]) * (rows[0] + columns[0] + 1) / 2 + rows[0] + 1) as f64,
		((rows[1] + columns[1]) * (rows[1] + columns[1] + 1) / 2 + rows[1] + 1) as f64,
	]
}

#[inline(never)]
fn divide_and_add(
	source: &[f64; 2],
	first_denominators: &[f64; 2],
	second_denominators: &[f64; 2],
	first_sums: &mut [f64; 2],
	second_sums: &mut [f64; 2],
) {
	first_sums[0] += source[0] / first_denominators[0];
	first_sums[1] += source[1] / first_denominators[1];
	second_sums[0] += source[0] / second_denominators[0];
	second_sums[1] += source[1] / second_denominators[1];
}

fn multiply<Denominators>(
	source: &[[f64; 2]],
	destination: &mut [[f64; 2]],
	denominators: Denominators,
) where
	Denominators: Fn([usize; 2], [usize; 2]) -> [f64; 2],
{
	for (pair_index, destination_pair) in destination.iter_mut().enumerate() {
		let row = 2 * pair_index;
		let first_rows = [row; 2];
		let second_rows = [row + 1; 2];
		let mut first_sums = [0.0; 2];
		let mut second_sums = [0.0; 2];

		for (source_pair_index, source_pair) in source.iter().enumerate() {
			let column = 2 * source_pair_index;
			let columns = [column, column + 1];
			divide_and_add(
				source_pair,
				&denominators(first_rows, columns),
				&denominators(second_rows, columns),
				&mut first_sums,
				&mut second_sums,
			);
		}

		destination_pair[0] = first_sums[0] + first_sums[1];
		destination_pair[1] = second_sums[0] + second_sums[1];
	}
}

fn multiply_at_av(source: &[[f64; 2]], destination: &mut [[f64; 2]], temporary: &mut [[f64; 2]]) {
	multiply(source, temporary, matrix_denominators);
	multiply(temporary, destination, |rows, columns| {
		matrix_denominators(columns, rows)
	});
}

fn dot(left: &[[f64; 2]], right: &[[f64; 2]]) -> f64 {
	let products = left
		.iter()
		.zip(right)
		.map(|(left_pair, right_pair)| [left_pair[0] * right_pair[0], left_pair[1] * right_pair[1]])
		.fold([0.0; 2], |sums, product| {
			[sums[0] + product[0], sums[1] + product[1]]
		});
	products[0] + products[1]
}

#[unsafe(export_name = "benchmark")]
pub extern "C" fn benchmark(vector_length: u32) -> f64 {
	let vector_length = vector_length as usize;
	if vector_length == 0 || vector_length > MAXIMUM_VECTOR_LENGTH || vector_length % 2 != 0 {
		return f64::NAN;
	}
	let vector_pair_count = vector_length / 2;

	let mut vector_u = [[0.0; 2]; MAXIMUM_VECTOR_PAIR_COUNT];
	let mut vector_v = [[0.0; 2]; MAXIMUM_VECTOR_PAIR_COUNT];
	let mut temporary = [[0.0; 2]; MAXIMUM_VECTOR_PAIR_COUNT];
	for pair in &mut vector_u[..vector_pair_count] {
		*pair = [1.0; 2];
	}
	let vector_u = &mut vector_u[..vector_pair_count];
	let vector_v = &mut vector_v[..vector_pair_count];
	let temporary = &mut temporary[..vector_pair_count];

	for _ in 0..ITERATION_COUNT {
		multiply_at_av(vector_u, vector_v, temporary);
		multiply_at_av(vector_v, vector_u, temporary);
	}

	(dot(vector_u, vector_v) / dot(vector_v, vector_v)).sqrt()
}

#[cfg(test)]
mod tests {
	#[test]
	fn computes_known_result() {
		let difference = (super::benchmark(100) - 1.274_219_991).abs();
		assert!(difference < 0.000_000_001);
	}
}
