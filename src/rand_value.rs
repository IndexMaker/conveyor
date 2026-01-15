use crate::common::amount::Amount;
use rand::{distr::Uniform, prelude::*};

pub struct ValueGen {
    rng: ThreadRng,
    sampler: Uniform<u128>,
    scale: u8,
}

impl ValueGen {
    pub fn new(low: u128, high: u128, scale: u8) -> Self {
        Self {
            rng: rand::rng(),
            sampler: Uniform::new_inclusive(low, high).expect("ValueGen"),
            scale,
        }
    }

    pub fn next(&mut self) -> Amount {
        let value = self.rng.sample(self.sampler);
        Amount::from_u128_with_scale(value, self.scale)
    }
}
