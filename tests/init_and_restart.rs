use gtest::{Log, Program};
use pebbles_game_io::{DifficultyLevel, GameState, PebblesAction, PebblesEvent, PebblesInit};
use utils::{init_pebbles_game, init_system, PEBBLES_GAME_ID, USER_1, USER_2};

mod utils;

#[test]
fn invalid_pebbles_count_and_max_pebbles_per_turn() {
    let sys = init_system();
    let game = Program::current_with_id(&sys, PEBBLES_GAME_ID);
    assert!(game
        .send(
            USER_1,
            PebblesInit {
                difficulty: DifficultyLevel::Hard,
                pebbles_count: 1,
                max_pebbles_per_turn: 2,
            },
        )
        .main_failed());

    let sys = init_system();
    let game = Program::current_with_id(&sys, PEBBLES_GAME_ID);
    assert!(game
        .send(
            USER_1,
            PebblesInit {
                difficulty: DifficultyLevel::Hard,
                pebbles_count: 6,
                max_pebbles_per_turn: 3,
            },
        )
        .main_failed());
}

#[test]
fn process_turn_if_first_player_is_program() {
    let pebbles_count = 15;
    let sys = init_system();
    let game = init_pebbles_game(
        &sys,
        USER_2,
        DifficultyLevel::Hard,
        Some(pebbles_count),
        Some(5),
    );

    let GameState {
        pebbles_remaining, ..
    } = game.read_state::<GameState, u32>(0).unwrap();

    assert_ne!(pebbles_remaining, pebbles_count);
}

#[test]
fn reset_game_state() {
    let sys = init_system();
    let game = init_pebbles_game(&sys, USER_1, DifficultyLevel::Easy, Some(15), Some(5));

    let prev_state = game.read_state::<GameState, u32>(0).unwrap();

    let res = game.send(
        USER_1,
        PebblesAction::Restart {
            difficulty: DifficultyLevel::Hard,
            pebbles_count: 20,
            max_pebbles_per_turn: 2,
        },
    );
    assert!(res.contains(
        &Log::builder()
            .dest(USER_1)
            .payload(PebblesEvent::GameRestarted)
    ));

    let current_state = game.read_state::<GameState, u32>(0).unwrap();
    assert_ne!(prev_state, current_state);
}
