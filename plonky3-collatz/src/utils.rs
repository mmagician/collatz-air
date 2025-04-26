use p3_field::Field;
use p3_matrix::dense::RowMajorMatrix;

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

pub(crate) fn generate_collatz_trace<const N: usize, F: Field>(
    starting_value: u32,
) -> RowMajorMatrix<F> {
    let mut sequence = compute_collatz_sequence(starting_value);
    sequence.resize((sequence.len()).next_power_of_two(), 1);
    let mut values = Vec::with_capacity(N * sequence.len());
    for i in 0..sequence.len() {
        for j in 0..N {
            values.push(F::from_u32(sequence[i] >> j & 1));
        }
    }
    RowMajorMatrix::new(values, N)
}
