pub(crate) fn compute_collatz_sequence(n: u32) -> Vec<u32> {
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
