use rand::Rng;

/// A "fair" random number generator that implements the gambler's fallacy.
pub struct FairRandomTable {
    // Invariant: the sum of all weights is always
    // equal to the number of weights.
    weights: Vec<f32>,
}

impl FairRandomTable {
    /// Creates a new random table with `n` options.
    pub fn new(num_options: usize) -> Self {
        assert_ne!(num_options, 0, "0 options in fairly random table");
        Self {
            weights: vec![1.0; num_options],
        }
    }

    /// Samples the next random value.
    pub fn sample(&mut self) -> usize {
        let sum_weights = self.weights.len() as f32;

        let generated = rand::thread_rng().gen_range(0.0..sum_weights);

        let mut total_weight = 0.;
        let (option, &chosen_weight) = self
            .weights
            .iter()
            .enumerate()
            .find(|(_, &weight)| {
                total_weight += weight;
                total_weight >= generated
            })
            .expect("no values in table");

        // Distribute the chosen value's weight to other values,
        // then set it to zero.
        self.weights[option] = 0.;
        let num_weights = self.weights.len();
        for (i, weight) in self.weights.iter_mut().enumerate() {
            if i != option {
                *weight += chosen_weight / ((num_weights - 1) as f32);
            }
        }

        option
    }
}

#[cfg(test)]
mod tests {
    use super::FairRandomTable;

    #[test]
    fn print_values() {
        let num_options = 10;

        let mut table = FairRandomTable::new(num_options);

        for _ in 0..100 {
            println!("{}", table.sample());
        }
    }
}
