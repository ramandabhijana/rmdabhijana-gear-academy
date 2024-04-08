use gtest::{Program, System};
use pebbles_game_io::{DifficultyLevel, PebblesInit};

pub const PEBBLES_GAME_ID: u64 = 2;
#[allow(unused)]
pub const USER_1: u64 = 3;
#[allow(unused)]
pub const USER_2: u64 = 4;

#[cfg(test)]
pub fn init_system() -> System {
    let system = System::new();
    system.init_logger();
    system
}

#[cfg(test)]
pub fn init_pebbles_game(
    sys: &System,
    from: u64,
    difficulty: DifficultyLevel,
    pebbles_count: Option<u32>,
    max_pebbles_per_turn: Option<u32>,
) -> Program<'_> {
    let game = Program::current_with_id(sys, PEBBLES_GAME_ID);

    assert!(!game
        .send(
            from,
            PebblesInit {
                difficulty,
                pebbles_count: pebbles_count.unwrap_or(15),
                max_pebbles_per_turn: max_pebbles_per_turn.unwrap_or(5),
            },
        )
        .main_failed());

    game
}
