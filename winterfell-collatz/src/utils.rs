use winterfell::math::{fields::f128::BaseElement, FieldElement, ToElements};

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

// The PublicInputs type bound on the Air trait is required to implement the `ToElements` trait.
// Due to the orphan rule, we need to create a newtype to hold the inner array.
pub struct PublicInputs<const N: usize> {
    pub values: [BaseElement; N],
    pub steps_count: BaseElement,
}

impl<const N: usize> From<(u32, u32)> for PublicInputs<N> {
    fn from(value: (u32, u32)) -> Self {
        let mut first = [BaseElement::ZERO; N];
        for i in 0..N {
            first[i] = BaseElement::from((value.0 >> i) & 1);
        }
        PublicInputs {
            values: first,
            steps_count: BaseElement::from(value.1),
        }
    }
}

impl<const N: usize> ToElements<BaseElement> for PublicInputs<N> {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut elements = self.values.to_vec();
        elements.push(self.steps_count);
        elements
    }
}

/// Returns zero only when a = zero || a == one.
pub fn is_binary<E: FieldElement>(a: E) -> E {
    a * a - a
}
