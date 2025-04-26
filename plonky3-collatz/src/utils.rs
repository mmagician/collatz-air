use p3_field::Field;
use p3_matrix::dense::RowMajorMatrix;

/// Computes the Collatz sequence starting from n until it reaches 1
fn compute_collatz_sequence(n: u32) -> Vec<u32> {
    let mut sequence = Vec::new();
    let mut current = n;

    while current != 1 {
        sequence.push(current);
        if current % 2 == 0 {
            current = current / 2;
        } else {
            current = 3 * current + 1;
        }
    }
    sequence.push(1);
    sequence
}

/// Generates a trace matrix for the Collatz sequence
/// Each row represents a number in the sequence in binary form (LSB first)
/// Plus additional two columns for the step counter and a boolean flag indicating if the row is a transition row (1) or repeated/init row (0)
/// The matrix is padded to the next power of two with (the binary representation of) 1's
pub(crate) fn generate_collatz_trace<const N: usize, F: Field>(
    starting_value: u32,
) -> (RowMajorMatrix<F>, u32) {
    let mut sequence = compute_collatz_sequence(starting_value);
    let steps = sequence.len() - 1;
    sequence.resize((sequence.len()).next_power_of_two(), 1);
    let mut values = Vec::with_capacity(N * sequence.len());
    for i in 0..sequence.len() {
        for j in 0..N {
            values.push(F::from_u32(sequence[i] >> j & 1));
        }

        if i == 0 {
            values.push(F::from_u32(i as u32));
            values.push(F::from_bool(false));
        } else if i <= steps {
            values.push(F::from_u32(i as u32));
            values.push(F::from_bool(true));
        } else {
            values.push(F::from_u32(steps as u32));
            values.push(F::from_bool(false));
        }
    }
    (RowMajorMatrix::new(values, N + 2), steps as u32)
}
