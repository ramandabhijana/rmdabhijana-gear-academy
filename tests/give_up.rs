use gtest::Log;
use pebbles_game_io::{DifficultyLevel, GameState, PebblesAction, PebblesEvent, Player};
use utils::{init_pebbles_game, init_system, USER_2};

mod utils;

#[test]
fn set_program_as_winner() {
    let sys = init_system();

    let game = init_pebbles_game(&sys, USER_2, DifficultyLevel::Hard, None, None);

    let res = game.send(USER_2, PebblesAction::GiveUp);
    assert!(!res.main_failed());
    assert!(res.contains(
        &Log::builder()
            .dest(USER_2)
            .payload(PebblesEvent::Won(Player::Program))
    ));

    let GameState { winner, .. } = game.read_state::<GameState, u32>(0).unwrap();
    assert_eq!(winner, Some(Player::Program));
}
