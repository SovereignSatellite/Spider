#![cfg_attr(not(test), no_std)]
#![crate_type = "cdylib"]

// Adapted from fannkuchredux-rust-4 under ../LICENSE.txt.

#[cfg(not(test))]
#[panic_handler]
fn panic(_information: &core::panic::PanicInfo<'_>) -> ! {
	loop {}
}

const MAXIMUM_PERMUTATION_LENGTH: usize = 12;
const RESULT_CHECKSUM_SCALE: i32 = 128;
const WORKING_LENGTH: usize = 16;

#[unsafe(export_name = "benchmark")]
pub extern "C" fn benchmark(permutation_length: u32) -> i32 {
	let permutation_length = permutation_length as usize;
	if !(1..=MAXIMUM_PERMUTATION_LENGTH).contains(&permutation_length) {
		return i32::MIN;
	}

	let mut factorials = [1u32; WORKING_LENGTH];
	for index in 1..=permutation_length {
		factorials[index] = factorials[index - 1] * index as u32;
	}
	let permutation_count = factorials[permutation_length];

	let mut rotation_counts = [0i32; WORKING_LENGTH];
	let mut working_permutation = [0i32; WORKING_LENGTH];
	let mut permutation = [0i32; WORKING_LENGTH];
	for (value, destination) in permutation.iter_mut().enumerate() {
		*destination = value as i32;
	}

	let mut checksum = 0;
	let mut maximum_flip_count = 0;

	for permutation_index in 0..permutation_count {
		if permutation[0] > 0 {
			working_permutation.copy_from_slice(&permutation);
			let mut flip_count = 1;
			let mut first_value = permutation[0] as usize;
			while working_permutation[first_value] != 0 {
				let next_first_value =
					core::mem::replace(&mut working_permutation[first_value], first_value as i32);
				if first_value > 2 {
					working_permutation[1..first_value].reverse();
				}
				first_value = next_first_value as usize;
				flip_count += 1;
			}

			checksum += if permutation_index % 2 == 0 {
				flip_count
			} else {
				-flip_count
			};
			maximum_flip_count = core::cmp::max(maximum_flip_count, flip_count);
		}

		if permutation_index + 1 < permutation_count {
			let mut first_value = permutation[1];
			permutation[1] = permutation[0];
			permutation[0] = first_value;
			let mut rotation_index = 1;
			while rotation_counts[rotation_index] >= rotation_index as i32 {
				rotation_counts[rotation_index] = 0;
				rotation_index += 1;
				let next_first_value = permutation[1];
				permutation[0] = next_first_value;
				for destination_index in 1..rotation_index {
					permutation[destination_index] = permutation[destination_index + 1];
				}
				permutation[rotation_index] =
					core::mem::replace(&mut first_value, next_first_value);
			}
			rotation_counts[rotation_index] += 1;
		}
	}

	checksum * RESULT_CHECKSUM_SCALE + maximum_flip_count as i32
}

#[cfg(test)]
mod tests {
	#[test]
	fn computes_known_result() {
		assert_eq!(super::benchmark(7), 29_200);
	}
}
