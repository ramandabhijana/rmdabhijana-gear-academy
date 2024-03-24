#![no_std]

use gstd::{exec, msg, String, ToOwned};
use pebbles_game_io::{
    DifficultyLevel, GameState, PebblesAction, PebblesEvent, PebblesInit, Player,
};

static mut GAME: Option<GameState> = None;

#[gstd::async_main]
async fn main() {
    let action: PebblesAction = msg::load().expect("Invalid action payload");
    let game = unsafe { GAME.as_mut().expect("Game not initialized") };

    let result = process_handle(action, game).await;
    msg::reply(result, 0).expect("Unexpected invalid reply result");
}

async fn process_handle(
    action: PebblesAction,
    game: &mut GameState,
) -> Result<PebblesEvent, String> {
    match action {
        PebblesAction::Turn(user_remove_count) => {
            if user_remove_count < 1
                || user_remove_count > game.max_pebbles_per_turn
                || user_remove_count > game.pebbles_count
            {
                return Err("Invalid number of pebbles".to_owned());
            }

            let pebbles_remaining = game
                .pebbles_remaining
                .checked_sub(user_remove_count)
                .unwrap_or_default();

            game.pebbles_remaining = pebbles_remaining;

            if game.pebbles_remaining == 0 {
                game.winner = Some(Player::User);
                return Ok(PebblesEvent::Won(Player::User));
            }

            let program_remove_count = get_remove_count_for_difficulty(
                game.difficulty.clone(),
                game.max_pebbles_per_turn,
                game.pebbles_count,
            );

            let pebbles_remaining = game
                .pebbles_remaining
                .checked_sub(program_remove_count)
                .unwrap_or_default();

            game.pebbles_remaining = pebbles_remaining;

            match pebbles_remaining == 0 {
                true => {
                    game.winner = Some(Player::Program);
                    Ok(PebblesEvent::Won(Player::Program))
                }
                false => Ok(PebblesEvent::CounterTurn(program_remove_count)),
            }
        }
        PebblesAction::GiveUp => Ok(PebblesEvent::Won(Player::Program)),
        PebblesAction::Restart {
            difficulty,
            pebbles_count,
            max_pebbles_per_turn,
        } => {
            let new_game = create_game(difficulty, pebbles_count, max_pebbles_per_turn)?;
            unsafe { GAME = Some(new_game) }
            Ok(PebblesEvent::GameRestarted)
        }
    }
}

#[no_mangle]
extern "C" fn init() {
    let PebblesInit {
        difficulty,
        pebbles_count,
        max_pebbles_per_turn,
    } = msg::load::<PebblesInit>().expect("Can't decode pebble init");
    let game = create_game(difficulty, pebbles_count, max_pebbles_per_turn).unwrap();
    unsafe { GAME = Some(game) }
    msg::reply("Successfully initialized", 0).expect("Initialization failed");
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
) -> Result<GameState, String> {
    if max_pebbles_per_turn <= 1 || pebbles_count <= 2 {
        return Err("Invalid input for max_pebbles_per_turn or/and pebbles_count".to_owned());
    }
    if max_pebbles_per_turn >= (pebbles_count / 2) {
        return Err(
            "Invalid max_pebbles_per_turn. Must be less than half of the remaining pebbles"
                .to_owned(),
        );
    }
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

    Ok(GameState {
        first_player,
        pebbles_count,
        max_pebbles_per_turn,
        difficulty,
        pebbles_remaining,
        winner: None,
    })
}
