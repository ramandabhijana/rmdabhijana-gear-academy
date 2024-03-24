#![no_std]

use gstd::{exec, msg};
use pebbles_game_io::{DifficultyLevel, GameState, PebblesAction, PebblesInit, Player};

static mut GAME: Option<GameState> = None;

#[no_mangle]
extern "C" fn init() {
    let PebblesInit {
        difficulty,
        pebbles_count,
        max_pebbles_per_turn,
    } = msg::load::<PebblesInit>().expect("Can't decode pebble init");

    let first_player = match get_random_u32() % 2 == 0 {
        true => Player::Program,
        false => Player::User,
    };

    let mut pebbles_remaining = pebbles_count;
    if first_player == Player::Program {
        let remove_count = match difficulty {
            DifficultyLevel::Easy => get_random_u32() % max_pebbles_per_turn + 1,
            DifficultyLevel::Hard => {
                let remainder = (pebbles_count - 1) % (max_pebbles_per_turn + 1);
                if remainder == 0 {
                    // user will be in losing position
                    max_pebbles_per_turn
                } else {
                    remainder
                }
            }
        };
        pebbles_remaining -= remove_count;
    }

    unsafe {
        GAME = Some(GameState {
            first_player,
            pebbles_count,
            max_pebbles_per_turn,
            difficulty,
            pebbles_remaining,
            winner: None,
        })
    }
}

#[no_mangle]
extern "C" fn handle() {
    let action: PebblesAction = msg::load().expect("Error loading PebblesAction");
    let game = unsafe { GAME.as_mut().expect("Game not initialized") };
    match action {
        PebblesAction::Turn(_) => todo!(),
        PebblesAction::GiveUp => todo!(),
        PebblesAction::Restart {
            difficulty,
            pebbles_count,
            max_pebbles_per_turn,
        } => todo!(),
    }
}

#[no_mangle]
extern "C" fn state() {
    let game = unsafe { GAME.as_ref().expect("The contract is not initialized") };
    msg::reply(game, 0).expect("Failed to share state");
}

fn get_random_u32() -> u32 {
    let salt = msg::id();
    let (hash, _num) = exec::random(salt.into()).expect("get_random_u32(): random call failed");
    u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
}
