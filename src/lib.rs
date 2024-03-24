#![no_std]

use gstd::{exec, msg};
use pebbles_game_io::{
    DifficultyLevel, GameState, PebblesAction, PebblesEvent, PebblesInit, Player,
};

static mut GAME: Option<GameState> = None;

#[no_mangle]
extern "C" fn init() {
    let PebblesInit {
        difficulty,
        pebbles_count,
        max_pebbles_per_turn,
    } = msg::load::<PebblesInit>().expect("Can't decode pebble init");
    let game = create_game(difficulty, pebbles_count, max_pebbles_per_turn);
    unsafe { GAME = Some(game) }
    msg::reply("Successfully initialized", 0).expect("Initialization failed");
}

#[no_mangle]
extern "C" fn handle() {
    let action: PebblesAction = msg::load().expect("Error loading PebblesAction");
    let game = unsafe { GAME.as_mut().expect("Game not initialized") };
    match action {
        PebblesAction::Turn(user_remove_count) => {
            // TODO: validate action payload
            if game.pebbles_remaining - user_remove_count == 0 {
                msg::reply(PebblesEvent::Won(Player::User), 0)
                    .expect("Error in sending reply PebblesEvent::Won");
            } else {
                let program_remove_count = get_remove_count_for_difficulty(
                    game.difficulty.clone(),
                    game.max_pebbles_per_turn,
                    game.pebbles_count,
                );
                match game.pebbles_remaining - program_remove_count == 0 {
                    true => {
                        msg::reply(PebblesEvent::Won(Player::Program), 0)
                            .expect("Error in sending reply PebblesEvent::Won");
                    }
                    false => {
                        msg::reply(PebblesEvent::CounterTurn(program_remove_count), 0)
                            .expect("Error in sending reply PebblesEvent::Won");
                    }
                }
            }
        }
        PebblesAction::GiveUp => {
            msg::reply(PebblesEvent::Won(Player::Program), 0)
                .expect("Error in sending reply PebblesEvent::Won");
        }
        PebblesAction::Restart {
            difficulty,
            pebbles_count,
            max_pebbles_per_turn,
        } => {
            let game = create_game(difficulty, pebbles_count, max_pebbles_per_turn);
            unsafe { GAME = Some(game) }
            msg::reply("Successfully initialized", 0).expect("Initialization failed");
        }
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

fn get_remove_count_for_difficulty(
    difficulty: DifficultyLevel,
    max_pebbles_per_turn: u32,
    pebbles_count: u32,
) -> u32 {
    match difficulty {
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
    }
}

fn create_game(
    difficulty: DifficultyLevel,
    pebbles_count: u32,
    max_pebbles_per_turn: u32,
) -> GameState {
    // TODO: validate payload

    let first_player = match get_random_u32() % 2 == 0 {
        true => Player::Program,
        false => Player::User,
    };

    let mut pebbles_remaining = pebbles_count;
    if first_player == Player::Program {
        let remove_count = get_remove_count_for_difficulty(
            difficulty.clone(),
            max_pebbles_per_turn,
            pebbles_count,
        );
        pebbles_remaining -= remove_count;
    }

    GameState {
        first_player,
        pebbles_count,
        max_pebbles_per_turn,
        difficulty,
        pebbles_remaining,
        winner: None,
    }
}
