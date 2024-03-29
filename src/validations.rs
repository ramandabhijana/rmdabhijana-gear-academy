use super::PebblesGame;

impl PebblesGame {
    pub(crate) fn validate_turn_remove_count(&self, count: u32) {
        if count < 1 || count > self.0.max_pebbles_per_turn || count > self.0.pebbles_count {
            panic!("Invalid number of pebbles")
        }
    }

    pub(crate) fn validate_max_pebbles_per_turn_and_pebbles_count(
        max_pebbles_per_turn: u32,
        pebbles_count: u32,
    ) {
        if max_pebbles_per_turn <= 1 || pebbles_count <= 2 {
            panic!("Invalid input for max_pebbles_per_turn or/and pebbles_count")
        }
        if max_pebbles_per_turn >= (pebbles_count / 2) {
            panic!("Invalid max_pebbles_per_turn. Must be less than half of the remaining pebbles")
        }
    }
}
