use gtest::Log;
use pebbles_game_io::{DifficultyLevel, GameState, PebblesAction, PebblesEvent, Player};

use crate::utils::{init_pebbles_game, init_system, USER_1};

mod utils;

#[test]
fn invalid_user_remove_count() {
    let sys = init_system();
    let game = init_pebbles_game(&sys, USER_1, DifficultyLevel::Hard, Some(10), Some(2));

    let res = game.send(USER_1, PebblesAction::Turn(4));
    assert!(res.main_failed());
}

fn user_turn(difficulty: DifficultyLevel) {
    let pebbles_count = 15u32;
    let max_pebbles_per_turn = 5u32;
    let user_turn = 4u32;

    let sys = init_system();
    let game = init_pebbles_game(
        &sys,
        USER_1,
        difficulty,
        Some(pebbles_count),
        Some(max_pebbles_per_turn),
    );

    let GameState {
        pebbles_remaining: initial_peb_rem,
        ..
    } = game.read_state(0).unwrap();

    let res = game.send(USER_1, PebblesAction::Turn(user_turn));
    assert!(!res.main_failed());

    let GameState {
        pebbles_remaining: curr_peb_rem,
        ..
    } = game.read_state(0).unwrap();
    let counter_turn = initial_peb_rem - user_turn - curr_peb_rem;
    let expected_log = Log::builder()
        .dest(USER_1)
        .payload(PebblesEvent::CounterTurn(counter_turn));
    assert!(res.contains(&expected_log));
}

#[test]
fn user_turn_easy_difficulty() {
    user_turn(DifficultyLevel::Easy)
}

#[test]
fn user_turn_hard_difficulty() {
    user_turn(DifficultyLevel::Hard)
}

#[test]
fn user_win() {
    let pebbles_count = 6u32;
    let max_pebbles_per_turn = 2u32;
    let user_turn = 2u32;

    let sys = init_system();

    let game = init_pebbles_game(
        &sys,
        USER_1,
        DifficultyLevel::Easy,
        Some(pebbles_count),
        Some(max_pebbles_per_turn),
    );

    let res = game.send(USER_1, PebblesAction::Turn(user_turn));
    assert!(!res.main_failed());

    let res = game.send(USER_1, PebblesAction::Turn(user_turn));
    assert!(!res.main_failed());

    let expected_log = Log::builder()
        .dest(USER_1)
        .payload(PebblesEvent::Won(Player::User));
    assert!(res.contains(&expected_log));
}

#[test]
fn program_win() {
    let pebbles_count = 15u32;
    let max_pebbles_per_turn = 5u32;

    let sys = init_system();

    let game = init_pebbles_game(
        &sys,
        USER_1,
        DifficultyLevel::Hard,
        Some(pebbles_count),
        Some(max_pebbles_per_turn),
    );

    let GameState {
        pebbles_remaining: prev_pebbles_remaining,
        ..
    } = game.read_state(0).unwrap();

    let user_turn = 3u32;

    let res = game.send(USER_1, PebblesAction::Turn(user_turn));
    assert!(!res.main_failed());

    let GameState {
        pebbles_remaining: curr_pebbles_remaining,
        ..
    } = game.read_state(0).unwrap();

    assert!(res.contains(
        &Log::builder()
            .dest(USER_1)
            .payload(PebblesEvent::CounterTurn(
                prev_pebbles_remaining - user_turn - curr_pebbles_remaining
            ))
    ));

    let res = game.send(USER_1, PebblesAction::Turn(user_turn));
    assert!(!res.main_failed());
    assert!(res.contains(
        &Log::builder()
            .dest(USER_1)
            .payload(PebblesEvent::Won(Player::Program))
    ));
}
