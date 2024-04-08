#![no_std]

use gstd::{exec, msg};
use pebbles_game_io::{
    DifficultyLevel, GameState, PebblesAction, PebblesEvent, PebblesInit, Player,
};

mod validations;

fn reply(pebbles_event: PebblesEvent) {
    msg::reply(pebbles_event, 0).expect("Error during a replying with PebblesEvent");
}

#[cfg(not(test))]
fn get_random_u32() -> u32 {
    let salt = msg::id();
    let (hash, _num) = exec::random(salt.into()).expect("get_random_u32(): random call failed");
    u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
}

#[cfg(test)]
fn get_random_u32() -> u32 {
    let seed: [u8; 32] = msg::source().into();
    u32::from_le_bytes([seed[0], seed[1], seed[2], seed[3]]) % 2
}

static mut GAME: Option<PebblesGame> = None;

#[derive(Default, Clone)]
pub struct PebblesGame(GameState);

impl PebblesGame {
    pub fn new(pebbles_count: u32, max_pebbles_per_turn: u32, difficulty: DifficultyLevel) -> Self {
        PebblesGame::validate_max_pebbles_per_turn_and_pebbles_count(
            max_pebbles_per_turn,
            pebbles_count,
        );
        let first_player = match get_random_u32() % 2 == 0 {
            true => Player::Program,
            false => Player::User,
        };

        let mut pebbles_remaining = pebbles_count;
        if first_player == Player::Program {
            let remove_count = Self::get_program_remove_count(
                &difficulty,
                pebbles_remaining,
                max_pebbles_per_turn,
            );
            pebbles_remaining -= remove_count;
        }

        Self(GameState {
            first_player,
            pebbles_count,
            max_pebbles_per_turn,
            difficulty,
            pebbles_remaining,
            winner: None,
        })
    }
}

impl PebblesGame {
    pub fn turn(&mut self, remove_count: u32) {
        self.validate_turn_remove_count(remove_count);

        self.0.pebbles_remaining = self
            .0
            .pebbles_remaining
            .checked_sub(remove_count)
            .unwrap_or_default();

        if self.pebbles_remaining_empty() {
            self.0.winner = Some(Player::User);
            return reply(PebblesEvent::Won(Player::User));
        }

        let program_remove_count = PebblesGame::get_program_remove_count(
            &self.0.difficulty,
            self.0.pebbles_remaining,
            self.0.max_pebbles_per_turn,
        );

        self.0.pebbles_remaining = self
            .0
            .pebbles_remaining
            .checked_sub(program_remove_count)
            .unwrap_or_default();

        match self.pebbles_remaining_empty() {
            true => {
                self.0.winner = Some(Player::Program);
                reply(PebblesEvent::Won(Player::Program))
            }
            false => reply(PebblesEvent::CounterTurn(program_remove_count)),
        }
    }

    pub fn give_up(&mut self) {
        self.0.winner = Some(Player::Program);
        reply(PebblesEvent::Won(Player::Program))
    }

    pub fn restart(difficulty: DifficultyLevel, pebbles_count: u32, max_pebbles_per_turn: u32) {
        let new_game = PebblesGame::new(pebbles_count, max_pebbles_per_turn, difficulty);
        unsafe { GAME = Some(new_game) };
        reply(PebblesEvent::GameRestarted)
    }

    fn pebbles_remaining_empty(&self) -> bool {
        self.0.pebbles_remaining == 0
    }

    fn get_program_remove_count(
        difficulty: &DifficultyLevel,
        pebbles_remaining: u32,
        max_pebbles_per_turn: u32,
    ) -> u32 {
        match difficulty {
            DifficultyLevel::Easy => {
                (get_random_u32() % max_pebbles_per_turn + 1).min(max_pebbles_per_turn)
            }
            DifficultyLevel::Hard => {
                let remainder = pebbles_remaining % (max_pebbles_per_turn + 1);
                if remainder == 0 {
                    max_pebbles_per_turn
                } else {
                    remainder
                }
            }
        }
    }
}

#[no_mangle]
extern "C" fn handle() {
    let action: PebblesAction = msg::load().expect("Invalid action payload");
    let game = unsafe { GAME.as_mut().expect("Game not initialized") };

    match action {
        PebblesAction::Turn(user_remove_count) => game.turn(user_remove_count),
        PebblesAction::GiveUp => game.give_up(),
        PebblesAction::Restart {
            difficulty,
            pebbles_count,
            max_pebbles_per_turn,
        } => PebblesGame::restart(difficulty, pebbles_count, max_pebbles_per_turn),
    }
}

#[no_mangle]
extern "C" fn init() {
    let PebblesInit {
        difficulty,
        pebbles_count,
        max_pebbles_per_turn,
    } = msg::load().expect("Can't decode pebble init");
    let game = PebblesGame::new(pebbles_count, max_pebbles_per_turn, difficulty);
    unsafe { GAME = Some(game) }
    msg::reply("Successfully initialized", 0).expect("Initialization failed");
}

#[no_mangle]
extern "C" fn state() {
    let game = unsafe { GAME.take().expect("Unexpected error in taking state") };
    let game = game.0;
    msg::reply(game, 0).expect("Failed to share state");
}
