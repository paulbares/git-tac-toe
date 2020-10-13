use std::error::Error;
use open_ttt_lib::board::Position;
use std::env;

mod game;
mod tag;

const STATE_FILE_NAME: &'static str = "resources/current_game_state.json";
const GAME_FILE_NAME: &'static str = "README.md";

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let sha1 = &args[1];

    let mut game = game::Match::load_from_file(STATE_FILE_NAME);

    // Pick a position to place a mark. Positions are zero based.
    // An error result is returned if the position is outside the bounds
    // of the board, the position is already owned, or the game is over.
    if game.is_new_game() {
        game.do_move(get_position(sha1));
    } else {
        game.ai_move();
    }

    game.evaluate_state();
    game.save_to_file(STATE_FILE_NAME);
    game.write_to_markdown(GAME_FILE_NAME);

    Ok(())
}

fn get_position(sha1: &str) -> Position {
    let mut sum: u64 = 0;
    for c in sha1.chars() {
        sum = sum + u64::from_str_radix(&c.to_string()[..], 16).unwrap();
    }
    let index = sum % 9;
    Position { row: (index / 3) as i32, column: (index % 3) as i32 }
}
