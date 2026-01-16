use itertools::Itertools;
use rand::seq::IndexedRandom;

use crate::common::labels::Labels;

pub fn rand_pick_assets(assets: &Labels, index_size: usize) -> Labels {
    let mut rng = rand::rng();

    let chosen = assets
        .data
        .choose_multiple(&mut rng, index_size)
        .cloned()
        .sorted()
        .collect_vec();

    Labels::from_vec_u128(chosen)
}
